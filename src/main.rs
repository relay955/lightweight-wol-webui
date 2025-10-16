#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod db;
mod api;
mod error;
mod module;

#[macro_use] extern crate rocket;

use std::thread;
use std::io::Write;
use rocket::fairing::AdHoc;
use rocket::form::FromForm;
use rocket::fs::FileServer;
use rocket::http::Method;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::runtime::Runtime;
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;
use tray_item::{IconSource, TrayItem};
use crate::db::{create_tables, is_exist_tables, Db};

fn build_rocket() -> rocket::Rocket<rocket::Build> {
    let cors = CorsOptions::default()
        .allow_credentials(true)
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(vec![Method::Get, Method::Post, Method::Put, Method::Delete].into_iter().map(From::from).collect())
        .to_cors().unwrap();

    rocket::build()
        .mount("/api", routes![api::user_api::join,api::user_api::login, 
            api::user_api::profile, api::user_api::check_first_user,
            api::device_api::get_devices, api::device_api::create_device,
            api::device_api::update_device, api::device_api::delete_device,
            api::device_api::move_device, api::device_api::get_device,
        ])
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

fn main() {
    let rt = Runtime::new().expect("Failed to create runtime");

    let rocket_thread = thread::spawn(move || {
        rt.block_on(async {
            if let Err(e) = build_rocket().launch().await {
                eprintln!("Rocket failed: {}", e);
            }
        });
    });

    // 트레이 아이콘 생성
    let mut tray = TrayItem::new("Lightweight WOL WebUI", IconSource::Resource("exe-icon"))
        .expect("Failed to create tray");

    // Quit 메뉴 추가
    let _ = tray.add_menu_item("Quit", || {
        std::process::exit(0);
    });

    let _ = rocket_thread.join();
}
