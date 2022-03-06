use std::str::FromStr;

use crate::{
    errors::{self, NeoiotError},
    mqtt_client,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ServerToDevice {
    pub message_id: String,
    pub account_id: String,
    pub device_id: String,
    pub command: String,
    pub is_sync: bool,
    pub ttl: Option<usize>,
}

impl ServerToDevice {
    pub fn new(
        account_id: &str,
        device_id: &str,
        command: &str,
        is_sync: bool,
        ttl: Option<usize>,
    ) -> Self {
        let message_id = xid::new().to_string();
        Self {
            message_id,
            is_sync,
            account_id: account_id.to_string(),
            device_id: device_id.to_string(),
            command: command.to_string(),
            ttl,
        }
    }
    pub fn topic(&self) -> String {
        let mode = if self.is_sync { "sync" } else { "async" };
        let topic = format!(
            "s2d/{}/{}/{}/{}/{}",
            self.account_id, self.device_id, self.command, mode, self.message_id,
        );
        if let Some(ttl) = self.ttl {
            return format!("{}/{}", topic, ttl);
        }
        topic
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerToDeviceBatch {
    pub message_id: String,
    pub account_id: String,
    pub label: String,
    pub command: String,
    pub ttl: Option<usize>,
}

impl ServerToDeviceBatch {
    pub fn new(account_id: &str, label: &str, command: &str, ttl: Option<usize>) -> Self {
        let message_id = xid::new().to_string();
        Self {
            message_id,
            account_id: account_id.to_string(),
            label: label.to_string(),
            command: command.to_string(),
            ttl,
        }
    }

    pub fn topic(&self) -> String {
        let topic = format!(
            "s2l/{account_id}/{label}/{command}/{message_id}/{}",
            account_id = self.account_id,
            label = self.label,
            command = self.command,
            message_id = self.message_id,
        );
        if let Some(ttl) = self.ttl {
            return format!("{}/{}", topic, ttl);
        }
        topic
    }
}

pub struct ACLRules {
    account_id: String,
    device_id: String,
}

impl ACLRules {
    pub fn new(account_id: String, device_id: String) -> Self {
        Self {
            account_id,
            device_id,
        }
    }
    pub fn sub_s2d(&self) -> String {
        // server to device
        format!(
            "s2d/{account_id}/{device_id}/+/+/+/#",
            account_id = self.account_id,
            device_id = self.device_id
        )
    }
    pub fn sub_s2l(&self) -> String {
        // server to lable
        format!("s2l/{account_id}/+/+/+/#", account_id = self.account_id)
    }
    pub fn sub_d2d(&self) -> String {
        // device to device
        format!(
            "d2d/{account_id}/{receiver_id}/+/+",
            account_id = self.account_id,
            receiver_id = self.device_id
        )
    }
    pub fn pub_s2dr(&self) -> String {
        // server to device response
        format!(
            "s2dr/{account_id}/{device_id}/+/+/+/#",
            account_id = self.account_id,
            device_id = self.device_id
        )
    }
    pub fn pub_d2d(&self) -> String {
        // device to device
        format!(
            "d2d/{account_id}/+/{sender_id}/+",
            account_id = self.account_id,
            sender_id = self.device_id
        )
    }
    pub fn pub_d2s(&self) -> String {
        // device to server
        format!(
            "d2s/{account_id}/{device_id}/+/+",
            account_id = self.account_id,
            device_id = self.device_id
        )
    }

    pub fn pub_metrics(&self) -> String {
        format!(
            "metrics/{account_id}/{device_id}/+",
            account_id = self.account_id,
            device_id = self.device_id
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Message {
    topic: Topics,
    payload: String,
}

impl Message {
    pub fn new(topic: Topics, payload: String) -> Self {
        Self { topic, payload }
    }
    pub async fn publish(&self, qos: u8) -> Result<(), NeoiotError> {
        mqtt_client::Client::send_command(self.topic.topic(), self.payload.clone(), qos).await
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Topics {
    S2D(ServerToDevice),
    S2L(ServerToDeviceBatch),
}

impl Topics {
    fn topic(&self) -> String {
        match self {
            Topics::S2D(cmd) => cmd.topic(),
            Topics::S2L(cmd) => cmd.topic(),
        }
    }
}

impl FromStr for Topics {
    type Err = errors::NeoiotError;
    fn from_str(topic: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = topic.split('/').collect();
        let message = match &parts[..] {
            ["s2d", account_id, device_id, command, mode, message_id] => {
                Topics::S2D(ServerToDevice {
                    message_id: message_id.to_string(),
                    account_id: account_id.to_string(),
                    device_id: device_id.to_string(),
                    command: command.to_string(),
                    is_sync: *mode == "sync",
                    ttl: None,
                })
            }
            ["s2d", account_id, device_id, command, mode, message_id, ttl] => {
                Topics::S2D(ServerToDevice {
                    message_id: message_id.to_string(),
                    account_id: account_id.to_string(),
                    device_id: device_id.to_string(),
                    command: command.to_string(),
                    is_sync: *mode == "sync",
                    ttl: ttl.parse().ok(),
                })
            }
            _ => return Err(NeoiotError::InvalidTopic(topic.to_string())),
        };
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic() {
        let topic = "s2d/test_account/test_device/test_command/sync/test_message_id";
        assert_eq!(
            topic.parse::<Topics>().unwrap(),
            Topics::S2D(ServerToDevice {
                message_id: "test_message_id".to_string(),
                account_id: "test_account".to_string(),
                device_id: "test_device".to_string(),
                command: "test_command".to_string(),
                is_sync: true,
                ttl: None,
            })
        );
        let topic = "s2d/test_account/test_device/test_command/async/test_message_id/3600";
        assert_eq!(
            topic.parse::<Topics>().unwrap(),
            Topics::S2D(ServerToDevice {
                message_id: "test_message_id".to_string(),
                account_id: "test_account".to_string(),
                device_id: "test_device".to_string(),
                is_sync: false,
                command: "test_command".to_string(),
                ttl: Some(3600),
            })
        );
        let topic = "cmd/test_account/test_device/test_command/sync/test_message_id/3600/fake";
        assert!(topic.parse::<Topics>().is_err(),);
        let topic = "blablabla/test_account/test_device/async/test_command/test_message_id/3600";
        assert!(topic.parse::<Topics>().is_err(),);
    }
}
