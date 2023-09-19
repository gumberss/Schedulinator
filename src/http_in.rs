use actix_web::{get, post, web, HttpResponse, Responder};

use deadpool_redis::{
    redis::{cmd, FromRedisValue},
    Config, Runtime,
};

#[get("/")]
async fn hello(data: web::Data<crate::schemas::components::AppComponents>) -> impl Responder {
    let mut conn = data.redis_pool.get().await.unwrap();

    cmd("SET")
        .arg(&["teste", "123"])
        .query_async::<_, ()>(&mut conn)
        .await
        .unwrap();

    let mut conn = data.redis_pool.get().await.unwrap();
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
    HttpResponse::Ok().body(value_p_s + &value_r)
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
