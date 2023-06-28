use crate::message::{Command, SessionMessage};
use crate::server;
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
    pub dm_accepted: bool,
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

    fn send_syn(
        &mut self,
        recipient: usize,
        inviter_key: String,
        _: &mut <WsSession as Actor>::Context,
    ) {
        self.addr.do_send(server::Syn {
            id: self.id.to_owned(),
            recipient,
            inviterKey: inviter_key,
        })
    }

    fn send_synack(
        &mut self,
        inviter_key: String,
        recipient_key: String,
        recipient: usize,
        _: &mut <WsSession as Actor>::Context,
    ) {
        self.recipient = recipient;
        self.addr.do_send(server::SynAck {
            id: self.id.to_owned(),
            inviterKey: inviter_key,
            recipientKey: recipient_key,
            recipient: self.recipient.to_owned(),
        })
    }

    fn send_ack(
        &mut self,
        recipient_key: String,
        recipient: usize,
        _: &mut <WsSession as Actor>::Context,
    ) {
        self.recipient = recipient;
        self.addr.do_send(server::Ack {
            id: self.id.to_owned(),
            recipientKey: recipient_key,
            recipient: self.recipient.to_owned(),
        })
    }

    fn send_message(&mut self, message: String, _: &mut <WsSession as Actor>::Context) {
        self.addr.do_send(server::ClientMessage {
            sender: self.id.to_owned(),
            msg: message,
            recipient: self.recipient.to_owned(),
        });
    }

    fn reset_id(&mut self, ctx: &mut <WsSession as Actor>::Context) {
        self.recipient = 0;
        self.addr
            .send(server::ResetID {
                id: self.id.to_owned(),
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
}

impl Handler<server::ServerMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: server::ServerMessage, ctx: &mut Self::Context) -> Self::Result {
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
                    Command::Message { message } => {
                        self.send_message(message, ctx);
                    }
                    Command::Syn {
                        inviterKey,
                        recipient,
                    } => {
                        let recipient_id: usize = recipient.parse().unwrap_or(0);
                        self.send_syn(recipient_id, inviterKey, ctx);
                    }
                    Command::SynAck {
                        inviterKey,
                        recipientKey,
                        recipient,
                    } => {
                        let recipient_id: usize = recipient.parse().unwrap_or(0);
                        self.send_synack(inviterKey, recipientKey, recipient_id, ctx);
                    }
                    Command::Ack {
                        recipientKey,
                        recipient,
                    } => {
                        let recipient_id: usize = recipient.parse().unwrap_or(0);
                        self.send_ack(recipientKey, recipient_id, ctx);
                    }
                    Command::Unknown => {}
                    Command::ResetId { id } => {
                        if id != self.id.to_string() {
                            return;
                        }
                        self.reset_id(ctx);
                    }
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
