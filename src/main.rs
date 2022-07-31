use actix_web::{
    error, get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use chrono::prelude::*;
use entity::mid_table;
use entity::mid_table::Entity as MidTable;
use futures::StreamExt;
use migration::{Migrator, MigratorTrait};
use rand::Rng;
use sea_orm::DatabaseConnection;
use sea_orm::{entity::*, query::*};
use serde::Deserialize;
use std::env;

enum AppError {
    ErrorFromDb,
    MoreThanDouble,
}

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[derive(Deserialize, Debug, Clone)]
struct Info {
    my_ip: String,
    my_mac: String,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

async fn collision(conn: &DatabaseConnection, number: i32) -> Result<bool, AppError> {
    match MidTable::find()
        .filter(mid_table::Column::MeshId.eq(number))
        .all(conn)
        .await
    {
        Ok(x) => match x.len() {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(AppError::MoreThanDouble),
        },
        Err(x) => {
            println!("Error : {:?}", x);
            Err(AppError::ErrorFromDb)
        }
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("unknownCommand")
}

#[get("/gen")]
async fn gen(
    data: web::Data<AppState>,
    mut payload: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut rng = rand::thread_rng();
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // Generate random number without colission
    let final_nubmer = loop {
        let number: i32 = rng.gen_range(100001..899999);
        match collision(&data.conn, number).await {
            Ok(false) => break number,
            Ok(true) => println!("Colissioned with {}", number),
            _ => {}
        }
    };

    let current_time = Utc::now();
    let dev_info = match serde_json::from_slice::<Info>(&body) {
        Ok(x) => x,
        Err(_) => Info {
            my_ip: match req.peer_addr() {
                Some(x) => x.to_string(),
                None => String::from("0.0.0.0"),
            },
            my_mac: String::from("ff:ff:ff:ff:ff:ff"),
        },
    };
    let dev_info_cloned = dev_info.clone();

    let db_res = mid_table::ActiveModel {
        mesh_id: Set(final_nubmer),
        claimed: Set(1),
        first_timestamp: Set(current_time),
        first_ip: Set(dev_info.my_ip.clone()),
        first_mac: Set(dev_info.my_mac.clone()),
        last_timestamp: Set(current_time),
        last_ip: Set(dev_info.my_ip),
        last_mac: Set(dev_info.my_mac),
        ..Default::default()
    }
    .save(&data.conn)
    .await;

    match db_res {
        Ok(_) => {
            // Move issue.
            println!(
                "New Mesh ID assigned. mid : {}, ip : {}, mac : {}",
                final_nubmer, dev_info_cloned.my_ip, dev_info_cloned.my_mac
            );
            Ok(HttpResponse::Ok().body(format!("{}", final_nubmer)))
        }
        Err(_) => {
            println!("Error with mid : {}", final_nubmer);
            Ok(HttpResponse::Ok().body("error"))
        }
    }
}

#[post("/claim/{id}")]
async fn claim(
    data: web::Data<AppState>,
    mut payload: web::Payload,
    req: HttpRequest,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    let dev_info = match serde_json::from_slice::<Info>(&body) {
        Ok(x) => x,
        Err(_) => Info {
            my_ip: match req.peer_addr() {
                Some(x) => x.to_string(),
                None => String::from("0.0.0.0"),
            },
            my_mac: String::from("ff:ff:ff:ff:ff:ff"),
        },
    };
    let dev_info_cloned = dev_info.clone();

    let matched_model = match MidTable::find()
        .filter(mid_table::Column::MeshId.eq(id.to_owned()))
        .all(&data.conn)
        .await
    {
        Ok(x) => match x.len() {
            0 => return Ok(HttpResponse::Ok().body("Not Found")),
            1 => x[0].clone(),
            _ => return Ok(HttpResponse::Ok().body("Internal Duplication Error")),
        },
        Err(_) => return Ok(HttpResponse::Ok().body("Internal Error")),
    };

    let new_claimed = matched_model.claimed + 1;
    let mut matched_model: mid_table::ActiveModel = matched_model.into();

    matched_model.claimed = Set(new_claimed);
    matched_model.last_timestamp = Set(Utc::now());
    matched_model.last_ip = Set(dev_info.my_ip);
    matched_model.last_mac = Set(dev_info.my_mac);

    let db_res = matched_model.update(&data.conn).await;

    match db_res {
        Ok(_) => {
            // Move issue.
            println!(
                "New Mesh ID claimed. mid : {}, ip : {}, mac : {}",
                id.to_owned(),
                dev_info_cloned.my_ip,
                dev_info_cloned.my_mac
            );
            Ok(HttpResponse::Ok().body(format!("Ok - Claimed : {}", new_claimed)))
        }
        Err(_) => {
            println!("Claim error. mid : {}", id.to_owned());
            Ok(HttpResponse::Ok().body("Internal Error"))
        }
    }
}

#[get("/check/{id}")]
async fn check(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    HttpResponse::Ok().body(match collision(&data.conn, id.into_inner()).await {
        Ok(false) => "notInUse",
        Ok(true) => "inUse",
        _ => "dbError",
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db_path = env::var("DATABASE_PATH").unwrap_or(format!(
        "{}/database.sqlite3",
        env::current_dir()?.to_str().unwrap()
    ));

    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    let conn = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    let state = AppState { conn };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(hello)
            .service(gen)
            .service(claim)
            .service(check)
    })
    .bind("127.0.0.1:7979")?
    .run()
    .await
}
