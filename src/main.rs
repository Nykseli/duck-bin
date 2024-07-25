use actix_files as fs;
use actix_web::{
    cookie::Cookie,
    get, post,
    web::{self},
    App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder,
};
use askama::Template;
use data::User;

mod data;
mod middleware;

#[derive(Template)]
#[template(path = "index.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

#[get("/")]
async fn hello(req: HttpRequest) -> impl Responder {
    if let Some(user) = req.extensions().get::<User>() {
        let hello = HelloTemplate { name: &user.name }.render().unwrap();
        HttpResponse::Ok().body(hello)
    } else {
        HttpResponse::TemporaryRedirect()
            .insert_header(("Location", "/login"))
            .body("")
    }
}

#[get("/login")]
async fn login_get() -> impl Responder {
    let login = LoginTemplate {}.render().unwrap();
    HttpResponse::Ok().body(login)
}

#[post("/login")]
async fn login_post() -> impl Responder {
    let mut resp = HttpResponse::SeeOther()
        .insert_header(("Location", "/"))
        .body("");
    resp.add_cookie(&Cookie::new("user_secret", "super-secret-key"))
        .unwrap();
    resp
}

async fn js_files(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let path: String = req.match_info().query("filename").parse().unwrap();
    let path = format!("./resources/js/{path}");
    let file = fs::NamedFile::open(path)?;
    Ok(file.use_last_modified(true))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::user::UserSession)
            .service(login_get)
            .service(login_post)
            .service(hello)
            .service(
                web::scope("/static").service(web::resource("js/{filename:.*.js}").to(js_files)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
