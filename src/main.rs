use actix_web::{web, App, HttpServer};

use http_in::*;

mod adapters;
mod components;
mod diplomat;
mod http_in;
mod logic;
mod schemas;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(schemas::components::AppComponents {
                app_name: String::from("Schedulinator"),
                redis_pool: components::redis::configure(),
                postgress_pool: components::postgress::configure(),
            }))
            .service(register)
            .service(echo)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
