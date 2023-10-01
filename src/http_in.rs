use serde_json;
use std::str::FromStr;

use actix_web::{get, post, web, HttpResponse, Responder};

use chrono::Utc;
use cron::Schedule;

use crate::adapters;
use crate::diplomat;
use crate::logic::task;
use crate::schemas::{components::AppComponents, wire_in};

#[get("/register")]
async fn register(
    data: web::Data<AppComponents>,
    payload: web::Json<wire_in::task::Task>,
) -> impl Responder {
    let task = adapters::wire_in::task::to_model(&payload);
    let is_valid = task::is_minimum_recurrence_time_valid(&task);

    if !is_valid {
        return HttpResponse::BadRequest().body(
            task::worst_case_retry(&task.retry_policy)
                .unwrap()
                .to_string(),
        );
    }
    let schedule = Schedule::from_str(&task.schedule);

    if schedule.is_err() {
        return HttpResponse::BadRequest().body(schedule.unwrap_err().to_string());
    }

    let next_execution_time = schedule.unwrap().upcoming(Utc).take(1).next();

    if next_execution_time.is_none() {
        return HttpResponse::BadRequest().body("There is no next execution for the task");
    }
    //todo: insert to the database
    let timestamp_next_execution_time = next_execution_time.unwrap().timestamp();

    let insert_cache_result = diplomat::redis::insert_task(
        timestamp_next_execution_time,
        &task,
        data.redis_pool.to_owned(),
    )
    .await;

    return match insert_cache_result {
        Err(_) => HttpResponse::BadRequest().body(format!("It wasn't possible to insert on Redis")),
        Ok(_) => HttpResponse::Ok().body("Ok"),
    };
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
