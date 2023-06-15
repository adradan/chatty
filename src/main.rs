use crate::message::TextMessage;
use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use dotenv::dotenv;
use session::WsSession;
use std::io;
use std::time::Instant;

mod message;
mod server;
mod session;

async fn get_chat(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    println!("{:?}", srv);
    ws::start(
        WsSession {
            id: 0,
            name: None,
            heartbeat: Instant::now(),
            recipient: 0,
            dm_accepted: false,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

async fn test() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();
    log::info!("Starting logger.");

    let server = server::ChatServer::new().start();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(server.clone()))
            .route("/ws/", web::get().to(get_chat))
            .route("/", web::get().to(test))
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
