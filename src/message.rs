use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Syn {
        inviterKey: String,
        recipient: String,
    },
    SynAck {
        inviterKey: String,
        recipientKey: String,
        recipient: String,
    },
    Ack {
        recipientKey: String,
        recipient: String,
    },
    Message {
        message: String,
    },
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
enum Status {
    OK,
    NotFound,
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
    status: Status,
    message: String,
}

impl SessionMessage {
    pub fn ok(msg: String) -> Self {
        SessionMessage {
            status: Status::OK,
            message: msg,
        }
    }

    pub fn no_recipient(msg: String) -> Self {
        SessionMessage {
            status: Status::NotFound,
            message: msg,
        }
    }
}
