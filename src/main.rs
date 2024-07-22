use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[get("/")]
async fn hello() -> impl Responder {
    let hello = HelloTemplate { name: "quack" }.render().unwrap();
    HttpResponse::Ok().body(hello)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
