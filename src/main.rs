use actix_files as fs;
use actix_web::{
    cookie::Cookie,
    get,
    http::header::ContentType,
    post,
    web::{self},
    App, Error, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder,
};
use askama::Template;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeDelta, Timelike, Utc};
use data::{Content, DataPool, User};
use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;

mod data;
mod middleware;
mod util;

#[cfg(feature = "hot_reload")]
mod hot_reload;
#[cfg(feature = "hot_reload")]
pub const HOT_RELOAD: bool = true;
#[cfg(not(feature = "hot_reload"))]
pub const HOT_RELOAD: bool = false;

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

    let user_secret = util::rand_string();

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

#[derive(Deserialize)]
struct ContentData {
    content: String,
    title: String,
    expire: String,
}

fn utc_to_ndt(utc: DateTime<Utc>) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(utc.year(), utc.month(), utc.day())
        .unwrap()
        .and_hms_opt(utc.hour(), utc.minute(), utc.second())
        .unwrap()
}

#[post("/add_content")]
async fn add_content(
    req: HttpRequest,
    web::Form(form): web::Form<ContentData>,
    db: web::Data<DataPool>,
) -> impl Responder {
    let user = {
        let extensions = req.extensions();
        // TODO: redirect to /login if no user
        if let Some(user) = extensions.get::<User>() {
            user.clone()
        } else {
            let login = LoginTemplate {}.render().unwrap();
            return HttpResponse::Unauthorized().body(login);
        }
    };

    let content_id = util::rand_string();
    let expire_hours = if form.expire == "30d" {
        Some(30 * 24)
    } else if form.expire == "7d" {
        Some(7 * 24)
    } else if form.expire == "1d" {
        Some(24)
    } else if form.expire == "1h" {
        Some(1)
    } else {
        None
    };

    // apparently sqlx/sqlite is not happy with DateTime so we need to some type conversion
    let created = chrono::Utc::now();
    let created_ndt = utc_to_ndt(created);
    let expires = expire_hours.map(|h| utc_to_ndt(created + TimeDelta::hours(h)));

    sqlx::query!(
        "INSERT INTO content (user_id, content_id, content, title, created, expires) VALUES (?, ?, ?, ?, ?, ?)",
        user.id,
        content_id,
        form.content,
        form.title,
        created_ndt,
        expires
    )
    .execute(&db.pool)
    .await
    .unwrap();

    HttpResponse::SeeOther()
        .insert_header(("Location", format!("/content/{content_id}")))
        .body("")
}

#[derive(Template)]
#[template(path = "content.html")]
struct ContentTemplate<'a> {
    content: &'a str,
}

#[get("/content/{id}")]
async fn get_content(path: web::Path<String>, db: web::Data<DataPool>) -> impl Responder {
    let content_id = path.into_inner();

    let content = sqlx::query_as!(
        Content,
        "SELECT * FROM content WHERE content_id=?",
        content_id
    )
    .fetch_one(&db.pool)
    .await;

    // TODO: handle 404
    let content = content.unwrap();

    let cont = ContentTemplate {
        content: &content.content,
    }
    .render()
    .unwrap();
    HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body(cont)
}

async fn js_files(req: HttpRequest) -> Result<fs::NamedFile, Error> {
    let path: String = req.match_info().query("filename").parse().unwrap();
    let path = format!("./resources/js/{path}");
    let file = fs::NamedFile::open(path)?;
    Ok(file.use_last_modified(true))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const DATABASE_PATH: &str = std::env!("DATABASE_URL");

    let sqlite_pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(DATABASE_PATH)
        .await
        .unwrap();
    let app_state = DataPool { pool: sqlite_pool };

    HttpServer::new(move || {
        let app = App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::user::UserSession)
            .service(login_get)
            .service(login_post)
            .service(add_content)
            .service(get_content)
            .service(hello)
            .service(
                web::scope("/static").service(web::resource("js/{filename:.*.js}").to(js_files)),
            );

        #[cfg(feature = "hot_reload")]
        {
            app.service(web::scope("/ws").route("/", web::get().to(hot_reload::hot_reload_ws)))
        }
        #[cfg(not(feature = "hot_reload"))]
        {
            app
        }
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
