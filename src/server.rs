use crate::server::MessageStatus::NewMessage;
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Deserialize, Serialize, Debug)]
pub enum MessageStatus {
    NoRecipient,
    UserJoined,
    NewMessage,
    MessageSent,
    StartedSession,
    Success,
}

#[derive(Message, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
pub struct Message {
    pub sender: usize,
    pub message: String,
    pub status: MessageStatus,
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct JoinDM {
    pub id: usize,
    pub recipient: usize,
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
    pub recipient: usize,
}

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rng: ThreadRng,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }

    fn send_message(&self, recipient: usize, message: &str, sender: usize, status: MessageStatus) {
        if let Some(r) = self.sessions.get(&recipient) {
            r.do_send(Message {
                sender,
                message: message.to_owned(),
                status,
            });
        }
    }

    fn recipient_exists(&self, recipient: &usize) -> bool {
        self.sessions.get(recipient).is_some()
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = usize;

    //noinspection RsBorrowChecker
    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);
        self.send_message(
            id.clone(),
            "Session created.",
            id.clone(),
            MessageStatus::StartedSession,
        );
        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.id);
    }
}

impl Handler<JoinDM> for ChatServer {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: JoinDM, _ctx: &mut Self::Context) -> Self::Result {
        let JoinDM { id, recipient } = msg;

        if self.recipient_exists(&recipient) {
            let message = format!("{id} has joined.");
            self.send_message(recipient, message.as_str(), id, MessageStatus::UserJoined);
            Ok(())
        } else {
            Err("Recipient not found.".to_string())
        }
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: ClientMessage, _: &mut Self::Context) -> Self::Result {
        if self.recipient_exists(&msg.recipient) {
            self.send_message(msg.recipient, msg.msg.as_str(), msg.id, NewMessage);
            Ok(())
        } else {
            Err("Recipient no longer has a session.".to_string())
        }
    }
}
