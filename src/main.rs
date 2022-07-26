// use actix_web::{
//     error, get, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
// };
use actix_web::{error, get, post, web, App, Error, HttpResponse, HttpServer, Responder};
use entity::idtable;
use entity::idtable::Entity as Post;
use futures_util::StreamExt;
use migration::{Migrator, MigratorTrait};
use rand::Rng;
use sea_orm::DatabaseConnection;
use sea_orm::{entity::*, query::*};
use serde::{Deserialize, Serialize};

use std::env;

const DEFAULT_POSTS_PER_PAGE: usize = 5;

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<usize>,
    idtables_per_page: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("unknownCommand")
}

#[get("/gen")]
async fn get_gen_normal() -> impl Responder {
    let mut rng = rand::thread_rng();
    let number: i32 = rng.gen_range(100001..899999);
    let contents = format!("{}", number);
    HttpResponse::Ok().body(contents)
}

#[get("/gen_test_err")]
async fn gen_test_err() -> impl Responder {
    HttpResponse::Ok().body(format!("ABCD00"))
}

#[post("/add")]
async fn post_add_normal(mut body: web::Payload) -> Result<HttpResponse, Error> {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        // Need to fix.
        bytes.extend_from_slice(&item?);
    }

    println!("Body {:?}!", bytes);
    Ok(HttpResponse::Ok().body(format!("ok")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .service(hello)
            .service(get_gen_normal)
            .service(post_add_normal)
            .service(gen_test_err)
    })
    .bind("127.0.0.1:7979")?
    .run()
    .await
}
