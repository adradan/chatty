use crate::message::{Command, SessionMessage};
use crate::server;
use crate::server::MessageStatus;
use actix::prelude::*;
use actix_web_actors::ws;
use actix_web_actors::ws::{Message, WebsocketContext};
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsSession {
    pub id: usize,
    pub name: Option<String>,
    pub heartbeat: Instant,
    pub recipient: usize,
    pub addr: Addr<server::ChatServer>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
        let addr = ctx.address();
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        act.id = res;
                    }
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr.do_send(server::Disconnect {
            id: self.id.to_owned(),
        });
        Running::Stop
    }
}

fn message_check(
    res: Result<Result<(), String>, MailboxError>,
    ctx: &mut WebsocketContext<WsSession>,
    sender: usize,
    success_status: MessageStatus,
    success_message: String,
    err_message: String,
) {
    // Something went horribly wrong with the chat server.
    if let Err(_) = res {
        ctx.stop();
        return;
    }
    match res.unwrap() {
        Ok(_) => {
            let message = server::Message {
                sender,
                message: success_message,
                status: success_status,
            };
            let response = serde_json::to_string(&message).unwrap();
            ctx.text(response);
        }
        Err(_) => {
            let message = server::Message {
                sender,
                message: err_message,
                status: MessageStatus::NoRecipient,
            };
            let response = serde_json::to_string(&message).unwrap();
            ctx.text(response);
        }
    }
}

impl WsSession {
    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.heartbeat) > TIMEOUT {
                log::info!("No heartbeat! Disconnecting.");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    fn join(&mut self, recipient: usize, ctx: &mut <WsSession as Actor>::Context) {
        self.recipient = recipient;
        self.addr
            .send(server::JoinDM {
                id: self.id.to_owned(),
                recipient: self.recipient.to_owned(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                let r = act.recipient.to_owned();
                let success_message = format!("{r}");
                let err_message = format!("Recipient ({r}) not found");
                message_check(
                    res,
                    ctx,
                    act.id,
                    MessageStatus::UserJoined,
                    success_message,
                    err_message,
                );
                fut::ready(())
            })
            .wait(ctx);
    }

    fn send_message(&mut self, message: String, ctx: &mut <WsSession as Actor>::Context) {
        self.addr
            .send(server::ClientMessage {
                id: self.id.to_owned(),
                msg: message,
                recipient: self.recipient.to_owned(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                let r = act.recipient.to_owned();
                let success_message = format!("Sent DM to: {r}");
                let err_message = format!("Recipient ({r}) not found");
                message_check(
                    res,
                    ctx,
                    act.id,
                    MessageStatus::MessageSent,
                    success_message,
                    err_message,
                );
                fut::ready(())
            })
            .wait(ctx);
    }
}

impl Handler<server::Message> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) -> Self::Result {
        let json_msg = serde_json::to_string(&msg).unwrap();
        ctx.text(json_msg);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            Message::Text(text) => {
                let text = text.trim();
                let command: Command =
                    serde_json::from_str(text).unwrap_or_else(|_| Command::Unknown);
                log::info!("New Text Message: {}", text);
                println!("{:?}", command);
                match command {
                    Command::Join { recipient } => {
                        self.join(recipient, ctx);
                    }
                    Command::Message { message } => {
                        self.send_message(message, ctx);
                    }
                    Command::Unknown => {}
                }
            }
            Message::Binary(_) => {
                log::debug!("Binary received.")
            }
            Message::Continuation(_) => {
                ctx.stop();
            }
            Message::Ping(msg) => {
                log::debug!("Received Ping.");
                self.heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Message::Pong(_) => {
                // log::debug!("Received Pong from {}.", self.id);
                self.heartbeat = Instant::now();
            }
            Message::Close(reason) => {
                log::info!("Closing WebSocket (ID: {}).", self.id);
                ctx.close(reason);
                ctx.stop();
            }
            Message::Nop => (),
        }
    }
}
