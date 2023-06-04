use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub enum Command {
    NoRecipient,
    ChatMessage,
    ChatMessageSent,
    MessageSent,
    StartedSession,
    Join,
    InviteDM,
    AcceptDM,
    RejectDM,
    Success,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Message {
    Join { recipient: usize },
    InviteDM { inviter: usize, recipient: usize },
    AcceptDM { inviter: usize, recipient: usize },
    RejectDM { inviter: usize, recipient: usize },
    ChatMessage { message: String },
    NoRecipient { recipient: usize },
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
pub struct InviteDM {
    pub inviter: usize,
    pub recipient: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AcceptDM {
    pub inviter: usize,
    pub recipient: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RejectDM {
    pub inviter: usize,
    pub recipient: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinDM {
    pub id: usize,
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
        self.send_message(sender, message, sender, Command::NoRecipient);
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

impl Handler<JoinDM> for ChatServer {
    // Inviter will send on being sent back the AcceptDM
    // Recipient will send upon sending AcceptDM
    type Result = ();

    fn handle(&mut self, msg: JoinDM, _ctx: &mut Self::Context) -> Self::Result {
        let JoinDM { id, recipient } = msg;

        if self.recipient_exists(&recipient) {
            let message = Message::Join { recipient };
            self.send_message(recipient, message, id, Command::Join);
            self.send_message(
                id,
                Message::String("DM Created".to_string()),
                id,
                Command::Join,
            );
        } else {
            self.send_no_recipient(id, recipient);
        }
    }
}

impl Handler<InviteDM> for ChatServer {
    // Comes from person that wants to invite someone to DM
    type Result = ();

    fn handle(&mut self, msg: InviteDM, ctx: &mut Self::Context) -> Self::Result {
        let InviteDM { inviter, recipient } = msg;

        if self.recipient_exists(&recipient) {
            let message = Message::InviteDM { inviter, recipient };
            self.send_message(recipient, message, inviter, Command::InviteDM);
        } else {
            self.send_no_recipient(inviter, recipient);
        }
    }
}

impl Handler<AcceptDM> for ChatServer {
    // Comes from person that is invited
    type Result = ();

    fn handle(&mut self, msg: AcceptDM, _: &mut Self::Context) -> Self::Result {
        let AcceptDM { inviter, recipient } = msg;

        if self.recipient_exists(&recipient) {
            let message = Message::AcceptDM { inviter, recipient };
            self.send_message(inviter, message, recipient, Command::AcceptDM);
        } else {
            self.send_no_recipient(inviter, recipient);
        }
    }
}

impl Handler<RejectDM> for ChatServer {
    // Comes from person that is invited
    type Result = ();

    fn handle(&mut self, msg: RejectDM, _: &mut Self::Context) -> Self::Result {
        let RejectDM { inviter, recipient } = msg;

        if self.recipient_exists(&recipient) {
            let message = Message::RejectDM { inviter, recipient };
            self.send_message(inviter, message, recipient, Command::RejectDM);
        } else {
            self.send_no_recipient(inviter, recipient);
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
            self.send_message(recipient, message, sender, Command::ChatMessage);
            self.send_message(
                sender,
                Message::String("Message sent.".to_string()),
                sender,
                Command::MessageSent,
            );
        } else {
            self.send_no_recipient(sender, recipient);
        }
    }
}
