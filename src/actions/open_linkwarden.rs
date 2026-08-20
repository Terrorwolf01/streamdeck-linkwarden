use crate::ActionSettings;
use openaction::{Action, ActionUuid, Instance, OpenActionResult, async_trait};

pub struct OpenLinkwarden;
#[async_trait]
impl Action for OpenLinkwarden {
    const UUID: ActionUuid = "at.terrorwolf.linkwarden.openlinkwarden";
    type Settings = ActionSettings;

    async fn key_down(&self, instance: &Instance, settings: &Self::Settings) -> OpenActionResult<()> {
        match Self::open(settings).await {
            Ok(()) => {
                let _ = instance.show_ok().await;
            }
            Err(error) => {
                log::error!("Failed to open Linkwarden: {:#}", error);
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

impl OpenLinkwarden {
    async fn open(settings: &ActionSettings) -> anyhow::Result<()> {
        let (_, instance_url) = crate::credentials(settings).await;
        if instance_url.is_empty() {
            anyhow::bail!("Instance URL is not set");
        }

        open::that(instance_url)?;

        Ok(())
    }
}