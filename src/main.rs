#[macro_use] extern crate rocket;

use rocket::fs::FileServer;
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(vec![Method::Get, Method::Post, Method::Put, Method::Delete].into_iter().map(From::from).collect())
        .to_cors().unwrap();

    rocket::build()
        // .mount("/api", routes![todos])
        .attach(cors)
        .mount("/", FileServer::from("./static"))
}