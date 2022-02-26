use std::str::FromStr;

use crate::{
    errors::{self, NeoiotError},
    mqtt_client,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub message_id: String,
    pub account_id: String,
    pub device_id: String,
    pub command: String,
    pub ttl: Option<usize>,
}

impl Command {
    pub fn new(account_id: &str, device_id: &str, command: &str, ttl: Option<usize>) -> Self {
        let message_id = xid::new().to_string();
        Self {
            message_id,
            account_id: account_id.to_string(),
            device_id: device_id.to_string(),
            command: command.to_string(),
            ttl,
        }
    }
    pub fn topic(&self) -> String {
        let topic = format!(
            "cmd/{}/{}/{}/{}",
            self.account_id, self.device_id, self.command, self.message_id,
        );
        if let Some(ttl) = self.ttl {
            return format!("{}/{}", topic, ttl);
        }
        topic
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
    Command(Command),
}
impl Topics {
    fn topic(&self) -> String {
        match self {
            Topics::Command(cmd) => cmd.topic(),
        }
    }
}

impl FromStr for Topics {
    type Err = errors::NeoiotError;
    fn from_str(topic: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = topic.split('/').collect();
        let message = match &parts[..] {
            // cmd/:account_id/:device_id/:command/:messageID
            ["cmd", account_id, device_id, command, message_id] => Topics::Command(Command {
                message_id: message_id.to_string(),
                account_id: account_id.to_string(),
                device_id: device_id.to_string(),
                command: command.to_string(),
                ttl: None,
            }),
            // cmd/:account_id/:device_id/:command/:messageID/:ttl
            ["cmd", account_id, device_id, command, message_id, ttl] => Topics::Command(Command {
                message_id: message_id.to_string(),
                account_id: account_id.to_string(),
                device_id: device_id.to_string(),
                command: command.to_string(),
                ttl: ttl.parse().ok(),
            }),
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
        let topic = "cmd/test_account/test_device/test_command/test_message_id";
        assert_eq!(
            topic.parse::<Topics>().unwrap(),
            Topics::Command(Command {
                message_id: "test_message_id".to_string(),
                account_id: "test_account".to_string(),
                device_id: "test_device".to_string(),
                command: "test_command".to_string(),
                ttl: None,
            })
        );
        let topic = "cmd/test_account/test_device/test_command/test_message_id/3600";
        assert_eq!(
            topic.parse::<Topics>().unwrap(),
            Topics::Command(Command {
                message_id: "test_message_id".to_string(),
                account_id: "test_account".to_string(),
                device_id: "test_device".to_string(),
                command: "test_command".to_string(),
                ttl: Some(3600),
            })
        );
        let topic = "cmd/test_account/test_device/test_command/test_message_id/3600/fake";
        assert!(topic.parse::<Topics>().is_err(),);
        let topic = "blablabla/test_account/test_device/test_command/test_message_id/3600";
        assert!(topic.parse::<Topics>().is_err(),);
    }
}
