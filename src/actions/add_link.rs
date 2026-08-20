use crate::ActionSettings;
use openaction::{Action, ActionUuid, Instance, OpenActionResult, async_trait};

pub struct AddLink;
#[async_trait]
impl Action for AddLink {
    const UUID: ActionUuid = "at.terrorwolf.linkwarden.addlink";
    type Settings = ActionSettings;

    async fn key_down(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        // Errors are logged and swallowed rather than returned: propagating an Err here would
        // surface as the action itself failing in the host UI over a single failed request,
        // when the action (and the plugin process) should keep working for the next key press.
        // show_ok/show_alert give the same signal directly on the key instead.
        match Self::send_link(settings).await {
            Ok(()) => {
                let _ = instance.show_ok().await;
            }
            Err(error) => {
                log::error!("Failed to add link: {:#}", error);
                let _ = instance.show_alert().await;
            }
        }

        Ok(())
    }

    async fn send_to_plugin(
        &self,
        instance: &Instance,
        settings: &Self::Settings,
        payload: &serde_json::Value,
    ) -> OpenActionResult<()> {
        crate::handle_connection_check(instance, settings, payload).await
    }
}

impl AddLink {
    /// Fetches `url` and pulls out its <title> text - the same text a browser would show as the
    /// tab name. Returns `None` rather than erroring if the page has no title, since that's a
    /// normal outcome (e.g. non-HTML URLs) that should just fall back to using the URL as-is.
    async fn fetch_page_title(url: &str) -> anyhow::Result<Option<String>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()?;
        let html = client.get(url).send().await?.text().await?;

        // Deliberately not a full HTML parser - to_ascii_lowercase() only rewrites ASCII bytes,
        // so byte offsets found in it stay valid for slicing the original (UTF-8) string. Doesn't
        // decode HTML entities (e.g. "&amp;" stays literal), which is an acceptable simplification
        // for a link name.
        let lower = html.to_ascii_lowercase();
        let Some(tag_start) = lower.find("<title") else {
            return Ok(None);
        };
        let Some(tag_end_offset) = lower[tag_start..].find('>') else {
            return Ok(None);
        };
        let content_start = tag_start + tag_end_offset + 1;
        let Some(close_offset) = lower[content_start..].find("</title") else {
            return Ok(None);
        };

        let title = html[content_start..content_start + close_offset].trim();
        Ok(if title.is_empty() { None } else { Some(title.to_string()) })
    }

    async fn send_link(settings: &ActionSettings) -> anyhow::Result<()> {
        let client = reqwest::Client::builder().build()?;
        let (token, instance_url) = crate::credentials(settings).await;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert("Accept", "application/json".parse()?);
        headers.insert("Authorization", format!("Bearer {}", token).parse()?);

        let url = arboard::Clipboard::new()?.get_text()?.trim().to_string();
        if url.is_empty() {
            anyhow::bail!("Clipboard is empty");
        }

        let name = if settings.use_page_title {
            match Self::fetch_page_title(&url).await {
                Ok(Some(title)) => title,
                Ok(None) => url.clone(),
                Err(error) => {
                    log::warn!("Failed to fetch page title, falling back to the URL: {:#}", error);
                    url.clone()
                }
            }
        } else {
            url.clone()
        };

        let data = r#"{
    "type": "url",
    "description": "Added via StreamDeck."
}"#;

        let mut json: serde_json::Value = serde_json::from_str(data)?;
        json["url"] = serde_json::Value::String(url);
        json["name"] = serde_json::Value::String(name);
        if !settings.description.is_empty() {
            json["description"] = serde_json::Value::String(settings.description.clone());
        }
        if !settings.tags.is_empty() {
            json["tags"] = serde_json::to_value(&settings.tags)?;
        }
        if let Some(collection_id) = settings.collection_id {
            json["collection"] = serde_json::json!({ "id": collection_id, "name": settings.collection_name });
        }

        let endpoint = format!("{}/api/v1/links", instance_url.trim_end_matches('/'));
        let request = client
            .request(reqwest::Method::POST, endpoint)
            .headers(headers)
            .json(&json);

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;

        log::debug!("Linkwarden response ({}): {}", status, body);

        if !status.is_success() {
            anyhow::bail!("Linkwarden returned {}: {}", status, body);
        }

        Ok(())
    }
}