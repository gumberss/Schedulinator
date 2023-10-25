use actix_web::{web, App, HttpServer};
use std::env;

use http_in::*;

mod adapters;
mod components;
mod controllers;
mod diplomat;
mod http_in;
mod logic;
mod schemas;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(schemas::components::AppComponents {
                app_name: String::from("Schedulinator"),
                redis_pool: components::redis::configure(),
                postgress_pool: components::postgress::configure(),
            }))
            .service(register)
            .service(echo)
            .service(execute_test)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
