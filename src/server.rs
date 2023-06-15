use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub enum Command {
    Syn,
    SynAck,
    Ack,
    NoRecipient,
    ChatMessage,
    ChatMessageSent,
    MessageSent,
    StartedSession,
    Success,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Message {
    Syn {
        inviterKey: String,
        recipient: usize,
    },
    SynAck {
        inviterKey: String,
        recipientKey: String,
        recipient: usize,
    },
    Ack {
        recipientKey: String,
        recipient: usize,
    },
    ChatMessage {
        message: String,
    },
    NoRecipient {
        recipient: usize,
    },
    String(String),
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        Message::ChatMessage { message: value }
    }
}

#[derive(Message, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
pub struct Session {
    pub server_message: ServerMessage,
    // ID of person they are DMing with
    pub accepted_dm: usize,
}

#[derive(Message, Deserialize, Serialize, Debug)]
#[rtype(result = "()")]
pub struct ServerMessage {
    pub sender: usize,
    pub message: Message,
    pub command: Command,
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<ServerMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Syn {
    pub id: usize,
    pub inviterKey: String,
    pub recipient: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SynAck {
    pub id: usize,
    pub inviterKey: String,
    pub recipientKey: String,
    pub recipient: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Ack {
    pub id: usize,
    pub recipientKey: String,
    pub recipient: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub sender: usize,
    pub msg: String,
    pub recipient: usize,
}

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<ServerMessage>>,
    rng: ThreadRng,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }

    fn send_message(&self, recipient: usize, message: Message, sender: usize, command: Command) {
        if let Some(r) = self.sessions.get(&recipient) {
            r.do_send(ServerMessage {
                sender,
                message,
                command,
            });
        }
    }

    fn recipient_exists(&self, recipient: &usize) -> bool {
        self.sessions.get(recipient).is_some()
    }

    fn send_no_recipient(&self, sender: usize, recipient: usize) {
        let message = Message::NoRecipient { recipient };
        self.send_message(sender.to_owned(), message, sender, Command::NoRecipient);
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
            id.to_owned(),
            Message::String("Session created.".to_string()),
            id.to_owned(),
            Command::StartedSession,
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

impl Handler<Syn> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Syn, _: &mut Self::Context) -> Self::Result {
        let Syn {
            id,
            inviterKey,
            recipient,
        } = msg;

        if self.recipient_exists(&recipient) {
            let message = Message::Syn {
                inviterKey,
                recipient: recipient.clone(),
            };
            self.send_message(recipient, message, id, Command::Syn);
        }
    }
}

impl Handler<SynAck> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: SynAck, _: &mut Self::Context) -> Self::Result {
        let SynAck {
            id,
            inviterKey,
            recipientKey,
            recipient,
        } = msg;

        if self.recipient_exists(&recipient) {
            let message = Message::SynAck {
                inviterKey,
                recipientKey,
                recipient: recipient.clone(),
            };
            self.send_message(recipient, message, id, Command::SynAck);
        }
    }
}

impl Handler<Ack> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Ack, _: &mut Self::Context) -> Self::Result {
        let Ack {
            id,
            recipientKey,
            recipient,
        } = msg;

        if self.recipient_exists(&recipient) {
            let message = Message::Ack {
                recipientKey,
                recipient: recipient.to_owned(),
            };
            self.send_message(recipient, message, id, Command::Ack);
        }
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Self::Context) -> Self::Result {
        let ClientMessage {
            sender,
            recipient,
            msg,
        } = msg;

        if self.recipient_exists(&recipient) {
            let message = Message::ChatMessage { message: msg };
            self.send_message(
                recipient.to_owned(),
                message,
                sender.to_owned(),
                Command::ChatMessage,
            );
            self.send_message(
                sender.to_owned(),
                Message::String("Message sent.".to_string()),
                sender,
                Command::MessageSent,
            );
        } else {
            self.send_no_recipient(sender, recipient);
        }
    }
}
