mod db;
mod api;
mod error;
mod jwt;
mod auth;

#[macro_use] extern crate rocket;

use std::str::Matches;
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::http::Method;
use rocket::serde::{Deserialize, Serialize};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;
use serde_json::ser::CharEscape::CarriageReturn;
use crate::db::{create_tables, is_exist_tables, Db};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    let cors = CorsOptions::default()
        .allow_credentials(true)
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(vec![Method::Get, Method::Post, Method::Put, Method::Delete].into_iter().map(From::from).collect())
        .to_cors().unwrap();

    rocket::build()
        .mount("/api", routes![api::user_api::join, api::user_api::login, api::user_api::profile])
        .register("/", catchers![api::catcher::unauthorized])
        .attach(Db::init()) // DB 풀 초기화
        .attach(AdHoc::try_on_ignite("Run DB Migrations", |rocket| async {
            let db = match Db::fetch(&rocket){
                Some(db) => db,
                None => return Err(rocket)
            };
            match is_exist_tables(&db).await {
                Ok(exist) => if exist {
                    println!("DB Already created.");
                    return Ok(rocket) 
                },
                Err(_) => return Err(rocket) 
            }
            
            if let Err(e) = create_tables(&db).await {
                return Err(rocket)
            }
            println!("DB created.");
            Ok(rocket)
        }))
        .attach(cors)
        .mount("/", FileServer::from("./static"))
}