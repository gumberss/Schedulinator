use serde::Serialize;
use actix_web::{ web, App, HttpServer};
mod http_in;
use http_in::*;

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                app_name: String::from("Schedulinator"),
            }))
            .service(hello)
            .service(echo)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}