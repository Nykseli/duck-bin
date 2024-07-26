use std::io::Read;

use actix_files as fs;
use actix_web::{
    cookie::Cookie,
    get, post,
    web::{self},
    App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder,
};
use askama::Template;
use data::{DataPool, User};
use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;

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

#[derive(Deserialize)]
struct LoginData {
    name: String,
    password: String,
}

#[post("/login")]
async fn login_post(
    web::Form(form): web::Form<LoginData>,
    db: web::Data<DataPool>,
) -> impl Responder {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE name=? and password=?",
        form.name,
        form.password
    )
    .fetch_one(&db.pool)
    .await;

    let user = match user {
        Ok(user) => user,
        Err(_) => {
            let login = LoginTemplate {}.render().unwrap();
            return HttpResponse::Unauthorized().body(login);
        }
    };

    let mut file_buf = [0u8; 64];
    let mut rand_file = std::fs::File::open("/dev/random").unwrap();
    let _ = rand_file.read(&mut file_buf).unwrap();
    let user_secret: Vec<u8> = file_buf
        .iter()
        .filter(|b| b.is_ascii_alphanumeric())
        .copied()
        .collect();
    let user_secret = String::from_utf8(user_secret).unwrap();

    sqlx::query!(
        "INSERT INTO user_sessions (user_id, session_id) VALUES (?, ?)",
        user.id,
        user_secret
    )
    .execute(&db.pool)
    .await
    .unwrap();

    let mut resp = HttpResponse::SeeOther()
        .insert_header(("Location", "/"))
        .body("");
    resp.add_cookie(&Cookie::new("user_secret", &user_secret))
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
    // TODO: make this configurable
    const DATABASE_PATH: &str = "tmp/users.db";

    let sqlite_pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(DATABASE_PATH)
        .await
        .unwrap();
    let app_state = DataPool { pool: sqlite_pool };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
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
