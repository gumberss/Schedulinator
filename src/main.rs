use actix_web::{web, App, HttpServer};
mod components;
mod http_in;
mod schemas;
use http_in::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(schemas::components::AppComponents {
                app_name: String::from("Schedulinator"),
                redis_pool: components::redis::configure(),
                postgress_pool: components::postgress::configure(),
            }))
            .service(hello)
            .service(echo)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
