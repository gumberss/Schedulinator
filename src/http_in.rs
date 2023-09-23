use serde_json;
use std::str::FromStr;

use actix_web::{get, post, web, HttpResponse, Responder};
use futures::future;

use deadpool_redis::{
    redis::{cmd, Value},
    Config, Runtime,
};

use chrono::Utc;
use cron::Schedule;

use crate::adapters;
use crate::logic::task;
use crate::schemas::{components::AppComponents, wire_in};

#[get("/register")]
async fn register(
    data: web::Data<AppComponents>,
    payload: web::Json<wire_in::task::Task>,
) -> impl Responder {
    let conn_future = data.redis_pool.get();
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
    let mut conn = conn_future.await.unwrap();

    let redis_task = adapters::wire_out::redis::task::to_dto(&task);

    let multi_result = cmd("MULTI").query_async::<_, ()>(&mut *conn).await;

    let zadd_result = cmd("ZADD")
        .arg(&[
            "schedules",
            &timestamp_next_execution_time.to_string(),
            //todo: change to id
            &task.name,
        ])
        .query_async::<_, ()>(&mut conn)
        .await;

    let set_result = cmd("SET")
        .arg(&[
            format!("task_{}", task.name.to_string()),
            serde_json::to_string(&redis_task).unwrap(),
        ])
        .query_async::<_, ()>(&mut conn)
        .await;

    let exec_results = cmd("EXEC").query_async::<_, ()>(&mut *conn).await;

    return match exec_results {
        Err(_) => HttpResponse::BadRequest().body(format!("It wasn't possible to insert on Redis")),
        /*  (Ok(_), Err(_)) | (Err(_), Ok(_)) => HttpResponse::BadRequest().body(format!(
            "Operation Partially Completed, but some error occured, resend the request"
        )), */
        Ok(_) => HttpResponse::Ok().body("Ok"),
    };

    /*  let mut conn = data.redis_pool.get().await.unwrap();
    let value_r: String = cmd("GET")
        .arg(&["teste"])
        .query_async(&mut conn)
        .await
        .unwrap();

     let mut conn_p = data.postgress_pool.get().await.unwrap();

    let stmt = conn_p
        .prepare_cached("SELECT id from schedules limit 1")
        .await
        .unwrap();
    let rows = conn_p.query(&stmt, &[]).await.unwrap();
    let value_p: i32 = rows[0].get(0);
    let value_p_s: String = value_p.to_string();
    HttpResponse::Ok().body(value_p_s + &value_r)*/
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
