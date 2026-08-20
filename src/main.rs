mod actions;
mod shared;
use actions::*;
use shared::logger;

use openaction::*;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::RwLock;

/// A Linkwarden tag or collection, as picked from the property inspector's selector. Shape
/// matches both the `/api/v1/tags` and `/api/v1/collections` responses closely enough to
/// deserialize directly from either, and matches what Linkwarden's `POST /api/v1/links` expects
/// for a tag/collection reference.
#[derive(Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ActionSettings {
    /// Per-action override; when non-empty, takes precedence over the global token.
    #[serde(rename = "token")]
    pub token: String,
    /// Per-action override; when non-empty, takes precedence over the global instance URL.
    #[serde(rename = "instanceUrl")]
    pub instance_url: String,
    /// When non-empty, overrides the fixed default description used for added links.
    #[serde(rename = "description")]
    pub description: String,
    /// Tags applied to added links, chosen from the property inspector's tag selector. Empty
    /// means no tags are applied.
    #[serde(rename = "tags")]
    pub tags: Vec<Tag>,
    /// Collection added links are filed into, chosen from the property inspector's collection
    /// selector. Unset means the Linkwarden default (Unorganized) is used.
    #[serde(rename = "collectionId")]
    pub collection_id: Option<i64>,
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    /// Opt-in: when true, the link's name is fetched from the target page's <title> instead of
    /// using the URL itself.
    #[serde(rename = "usePageTitle")]
    pub use_page_title: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct GlobalSettings {
    /// Default token used when an action doesn't set its own `ActionSettings::token`.
    #[serde(rename = "token")]
    pub token: String,
    /// Default instance URL used when an action doesn't set its own `ActionSettings::instance_url`.
    #[serde(rename = "instanceUrl")]
    pub instance_url: String,
}

pub fn current_global_settings() -> &'static RwLock<GlobalSettings> {
    static SETTINGS: OnceLock<RwLock<GlobalSettings>> = OnceLock::new();
    SETTINGS.get_or_init(|| RwLock::new(GlobalSettings::default()))
}

/// Resolves the effective API token and instance URL for an action instance: a non-empty
/// per-action override in `settings` takes precedence, otherwise falls back to the global
/// settings. Shared by every action, since all of them can target a different Linkwarden
/// instance than the global default.
pub async fn credentials(settings: &ActionSettings) -> (String, String) {
    let global = current_global_settings().read().await;
    let token = if !settings.token.is_empty() {
        settings.token.clone()
    } else {
        global.token.clone()
    };
    let instance_url = if !settings.instance_url.is_empty() {
        settings.instance_url.clone()
    } else {
        global.instance_url.clone()
    };
    (token, instance_url)
}

async fn fetch_tags(settings: &ActionSettings) -> anyhow::Result<Vec<Tag>> {
    let (token, instance_url) = credentials(settings).await;
    let client = reqwest::Client::builder().build()?;

    let endpoint = format!("{}/api/v1/tags", instance_url.trim_end_matches('/'));
    let response = client
        .get(endpoint)
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    let body: serde_json::Value = response.json().await?;
    let tags: Vec<Tag> = serde_json::from_value(body["data"]["tags"].clone())?;

    Ok(tags)
}

async fn fetch_collections(settings: &ActionSettings) -> anyhow::Result<Vec<Collection>> {
    let (token, instance_url) = credentials(settings).await;
    let client = reqwest::Client::builder().build()?;

    let endpoint = format!("{}/api/v1/collections", instance_url.trim_end_matches('/'));
    let response = client
        .get(endpoint)
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    let body: serde_json::Value = response.json().await?;
    let collections: Vec<Collection> = serde_json::from_value(body["response"].clone())?;

    Ok(collections)
}

/// Handles the getTags/getCollections round trip that every action's property inspector uses to
/// populate dropdowns and drive the connection-status indicator (via ConnectionSettings.svelte,
/// shared by every action's PI page). Every `Action` impl's `send_to_plugin` should delegate to
/// this - without it, a PI showing <ConnectionSettings /> sends the requests but never gets a
/// reply, since `send_to_plugin` is dispatched per-action and defaults to a no-op.
pub async fn handle_connection_check(
    instance: &Instance,
    settings: &ActionSettings,
    payload: &serde_json::Value,
) -> OpenActionResult<()> {
    match payload.get("event").and_then(|v| v.as_str()) {
        Some("getTags") => {
            let (tags, error) = match fetch_tags(settings).await {
                Ok(tags) => (tags, None),
                Err(error) => {
                    log::error!("Failed to fetch tags: {:#}", error);
                    (Vec::new(), Some(error.to_string()))
                }
            };
            instance
                .send_to_property_inspector(serde_json::json!({ "event": "tags", "tags": tags, "error": error }))
                .await?;
        }
        Some("getCollections") => {
            let (collections, error) = match fetch_collections(settings).await {
                Ok(collections) => (collections, None),
                Err(error) => {
                    log::error!("Failed to fetch collections: {:#}", error);
                    (Vec::new(), Some(error.to_string()))
                }
            };
            instance
                .send_to_property_inspector(
                    serde_json::json!({ "event": "collections", "collections": collections, "error": error }),
                )
                .await?;
        }
        _ => {}
    }

    Ok(())
}

pub struct GlobalSettingsHandler;
#[async_trait]
impl global_events::GlobalEventHandler for GlobalSettingsHandler {
    async fn plugin_ready(&self) -> OpenActionResult<()> {
        // Fire-and-forget: asks the host to send the current global settings. The response
        // arrives asynchronously as a did_receive_global_settings event, handled below.
        get_global_settings().await
    }

    async fn did_receive_global_settings(
        &self,
        event: global_events::DidReceiveGlobalSettingsEvent,
    ) -> OpenActionResult<()> {
        // Falls back to defaults rather than erroring out if the host sends something that
        // doesn't deserialize (e.g. settings saved by an older/incompatible plugin version).
        let settings: GlobalSettings =
            serde_json::from_value(event.payload.settings).unwrap_or_default();
        *current_global_settings().write().await = settings;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> OpenActionResult<()> {
    logger::init();

    global_events::set_global_event_handler(&GlobalSettingsHandler);
    register_action(add_link::AddLink).await;
    register_action(open_linkwarden::OpenLinkwarden).await;

    log::info!("Linkwarden plugin starting");

    // openaction::run() connects once and returns as soon as the websocket closes, even on a
    // clean disconnect (Ok(())), with no retry of its own. Since Stream Deck treats the process
    // exiting as an unexpected crash and disables the plugin, keep reconnecting instead of
    // letting main() return. run() is spawned rather than awaited directly so a panic inside it
    // (e.g. from a malformed event the crate doesn't expect) surfaces as a JoinError instead of
    // unwinding straight through main() and killing the whole process.
    let args: Vec<String> = std::env::args().collect();
    loop {
        let args = args.clone();
        match tokio::spawn(async move { run(args).await }).await {
            Ok(Ok(())) => {
                log::warn!("OpenAction connection closed cleanly, reconnecting");
            }
            Ok(Err(error)) => {
                log::error!("OpenAction connection ended with an error, reconnecting: {}", error);
            }
            Err(join_error) => {
                log::error!("OpenAction task panicked, reconnecting: {}", join_error);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}