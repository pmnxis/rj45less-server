// use actix_web::{
//     error, get, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
// };
use actix_web::{error, get, post, web, App, Error, HttpResponse, HttpServer, Responder};
use entity::mid_table;
use entity::mid_table::Entity as Post;
use migration::{Migrator, MigratorTrait, TableCreateStatement};
use rand::Rng;
use sea_orm::{entity::*, query::*};
use sea_orm::{DatabaseConnection, DbBackend};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::Path;

const DEFAULT_POSTS_PER_PAGE: usize = 5;

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<usize>,
    mid_table_per_page: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}

async fn collision_check_from_db(number: i32) -> bool {
    // todo!
    return true;
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("unknownCommand")
}

#[get("/gen")]
async fn gen() -> impl Responder {
    let mut rng = rand::thread_rng();
    let number: i32 = rng.gen_range(100001..899999);

    let contents = format!("{}", number);
    HttpResponse::Ok().body(contents)
}

#[get("/check/{id}")]
async fn check() -> impl Responder {
    HttpResponse::Ok().body("Ahoy")
}

// async fn setup_schema(db: &DbConn) {
//     let schema = Schema::new(DbBackend::Sqlite);

//     // Derive from Entity
//     let stmt: TableCreateStatement = schema.create_table_from_entity(mid_table);

//     // Execute create table statement
//     let result = db.execute(db.get_database_backend().build(&stmt)).await;
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_path = env::var("DATABASE_PATH").unwrap_or(format!(
        "{}/database.sqlite3",
        env::current_dir()?.to_str().unwrap()
    ));
    let db_url = format!("sqlite://{}", db_path);

    if !Path::new(&db_path).exists() {
        File::create(&db_path)?;
    }

    let conn = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    let state = AppState { conn };

    HttpServer::new(move || App::new().service(hello).service(gen).service(check))
        .bind("127.0.0.1:7979")?
        .run()
        .await
}
