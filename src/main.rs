use actix_files as fs;
use actix_web::{get, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
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

#[get("/js/{filename:.*.js}")]
async fn js_files(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let path: String = req.match_info().query("filename").parse().unwrap();
    let path = format!("./resources/js/{path}");
    let file = fs::NamedFile::open(path)?;
    Ok(file.use_last_modified(true))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(js_files))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
