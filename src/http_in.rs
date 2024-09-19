use actix_web::{get, post, web, HttpResponse, Responder};

use crate::adapters;
use crate::controllers;

use crate::schemas::{components::AppComponents, wire_in, wire_out};

#[post("/register")]
async fn register(
    data: web::Data<AppComponents>,
    payload: web::Json<wire_in::task::Task>,
) -> impl Responder {
    let task = adapters::wire_in::task::to_model(&payload);
    match controllers::insertion::insert(task, &data.redis_pool, &data.postgress_pool).await {
        Err(e) => HttpResponse::BadRequest().body(
            serde_json::to_string(&wire_out::http::error::Error {
                message: e.to_owned(),
            })
            .unwrap(),
        ),
        Ok(model) => HttpResponse::Ok()
            .body(serde_json::to_string(&adapters::wire_out::http::task::to_wire(&model)).unwrap()),
    }
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    dbg!("OLAAAAAA");
    HttpResponse::Ok().body(req_body)
}
