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
    let components = schemas::components::AppComponents {
        app_name: String::from("Schedulinator"),
        redis_pool: components::redis::configure(),
        postgress_pool: components::postgress::configure(),
    };

    let components_b = components.clone();
    actix_web::rt::spawn(async move { controllers::execution::execute_tasks(components_b).await });

    HttpServer::new(move || {
        let cp = components.clone();
        App::new()
            .app_data(web::Data::new(cp))
            .service(register)
            .service(echo)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
