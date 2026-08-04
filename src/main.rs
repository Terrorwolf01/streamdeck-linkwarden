mod actions;
mod shared;
use actions::*;
use shared::logger;

use openaction::*;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::RwLock;

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