use crate::ActionSettings;
use openaction::{Action, ActionUuid, Instance, OpenActionResult, async_trait};

pub struct AddLink;
#[async_trait]
impl Action for AddLink {
    const UUID: ActionUuid = "at.terrorwolf.linkwarden.addlink";
    type Settings = ActionSettings;

    async fn key_down(&self, _instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        // Errors are logged and swallowed rather than returned: propagating an Err here would
        // surface as the action itself failing in the host UI over a single failed request,
        // when the action (and the plugin process) should keep working for the next key press.
        if let Err(error) = Self::send_link(settings).await {
            log::error!("Failed to add link: {:#}", error);
        }

        Ok(())
    }
}

impl AddLink {
    async fn send_link(settings: &ActionSettings) -> anyhow::Result<()> {
        let client = reqwest::Client::builder().build()?;

        let (token, instance_url) = {
            let global = crate::current_global_settings().read().await;
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
        };

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert("Accept", "application/json".parse()?);
        headers.insert("Authorization", format!("Bearer {}", token).parse()?);

        // TODO: this is hardcoded placeholder data for testing against a specific Linkwarden
        // test account - it does not read the URL from the clipboard as advertised in
        // assets/manifest.json, and the tag/collection IDs below only exist on that account.
        let data = r#"{
    "name": "string",
    "url": "https://minefort.de",
    "type": "url",
    "description": "Added via StreamDeck.",
    "tags": [
        {
            "id": 3,
            "name": "test"
        }
    ],
    "collection": {
        "id": 12,
        "name": "Unorganized"
    }
}"#;

        let mut json: serde_json::Value = serde_json::from_str(data)?;
        if !settings.description.is_empty() {
            json["description"] = serde_json::Value::String(settings.description.clone());
        }

        let endpoint = format!("{}/api/v1/links", instance_url.trim_end_matches('/'));
        let request = client
            .request(reqwest::Method::POST, endpoint)
            .headers(headers)
            .json(&json);

        // Note: the response status is not checked here, so a non-2xx response (e.g. an
        // invalid token or malformed request) is not treated as a failure - it's only visible
        // in the debug log below, not surfaced to the caller as an Err.
        let response = request.send().await?;
        let body = response.text().await?;

        log::debug!("Linkwarden response: {}", body);

        Ok(())
    }
}