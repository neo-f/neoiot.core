use serde_json::json;

use crate::{config::SETTINGS, errors::NeoiotError};

pub struct Client;

impl Client {
    pub async fn send_command(topic: String, payload: String, qos: u8) -> Result<(), NeoiotError> {
        let config = &SETTINGS.emqx;
        let url = format!("{}/api/v4/mqtt/publish", config.management_host);
        let body = json!({
            "topic":    topic,
            "payload":  payload,
            "qos":      qos,
            "retain":   false,
            "encoding": "plain",
        });
        reqwest::Client::new()
            .post(url)
            .json(&body)
            .basic_auth(config.app_id.clone(), Some(config.app_secret.clone()))
            .send()
            .await
            .map_err(|e| NeoiotError::EmqxManagementError(e.to_string()))?;
        Ok(())
    }
}
