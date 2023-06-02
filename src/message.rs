use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Join { recipient: usize },
    Message { message: String },
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextMessage {
    pub command: Command,
}

impl TextMessage {
    pub fn default() -> Self {
        TextMessage {
            command: Command::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionMessage {
    status: String,
    message: String,
}

impl SessionMessage {
    pub fn ok(msg: String) -> Self {
        SessionMessage {
            status: "OK".to_string(),
            message: msg.to_string(),
        }
    }
}
