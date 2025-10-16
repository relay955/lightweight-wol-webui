#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod db;
mod api;
mod error;
mod module;
mod config;

#[macro_use] extern crate rocket;

use std::thread;
use std::io::Write;
use std::fs;
use rocket::fairing::AdHoc;
use rocket::form::FromForm;
use rocket::fs::FileServer;
use rocket::http::Method;
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::runtime::Runtime;
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_db_pools::Database;
use tray_item::{IconSource, TrayItem};
use crate::config::{CorsConfig, JwtConfig};
use crate::db::{create_tables, is_exist_tables, Db};
use crate::module::jwt::generate_random_secret;

fn update_secret_in_config(secret: &str) -> std::io::Result<()> {
    let config_path = "Rocket.toml";
    let content = fs::read_to_string(config_path)?;
    let updated_content = content.replace(r#"secret = "generate""#, &format!(r#"secret = "{}""#, secret));
    fs::write(config_path, updated_content)?;
    Ok(())
}

fn build_rocket() -> rocket::Rocket<rocket::Build> {
    let mut jwt_config = rocket::Config::figment()
        .extract_inner::<JwtConfig>("jwt")
        .unwrap_or_default();
    let cors_config = rocket::Config::figment()
        .extract_inner::<CorsConfig>("cors")
        .unwrap_or_default();

    if jwt_config.secret == "generate" {
        let generated_secret = generate_random_secret();
        if let Err(e) = update_secret_in_config(&generated_secret) {
            eprintln!("Warning: Failed to update Rocket.toml with generated secret: {}", e);
        }
        jwt_config.secret = generated_secret;
    }

    let allowed_origins = if cors_config.allow_origin == "*" {
        AllowedOrigins::all()
    } else {
        AllowedOrigins::some_exact(&[&cors_config.allow_origin])
    };
    
    let cors = CorsOptions::default()
        .allow_credentials(cors_config.allow_credentials)
        .allowed_origins(allowed_origins)
        .allowed_methods(vec![Method::Get, Method::Post, Method::Put, Method::Delete].into_iter().map(From::from).collect())
        .to_cors().unwrap();

    rocket::build()
        .manage(jwt_config)
        .mount("/api", routes![api::user_api::join,api::user_api::login, 
            api::user_api::profile, api::user_api::check_first_user,
            api::device_api::get_devices, api::device_api::create_device,
            api::device_api::update_device, api::device_api::delete_device,
            api::device_api::move_device, api::device_api::get_device,
            api::device_api::wake_device,
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
