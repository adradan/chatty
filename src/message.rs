use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Join { recipient: usize },
    InviteDM { recipient: usize },
    AcceptDM { recipient: usize },
    RejectDM { recipient: usize },
    Message { message: String },
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
