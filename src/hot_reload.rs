use actix::{Actor, StreamHandler};
use actix_web::{
    web::{self},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;

struct HotReloadWs;

impl Actor for HotReloadWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for HotReloadWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // Only needs to pong since it's only used to check for reconnections
        if let Ok(ws::Message::Ping(msg)) = msg {
            ctx.pong(&msg)
        }
    }
}

pub async fn hot_reload_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(HotReloadWs {}, &req, stream);
    println!("hot_reload.js connected");
    resp
}
