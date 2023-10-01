use std::process::Output;

use crate::adapters::wire_out;
use crate::schemas::models::task;
use deadpool_redis::{
    redis::{cmd, Value},
    Config, Pool, Runtime,
};
use futures::{future, Future};

pub async fn insert_task(
    timestamp_next_execution_time: i64,
    task: &task::Task,
    pool: Pool,
) -> Result<(), String> {
    let conn_future = pool.get();
    let mut conn = conn_future.await.unwrap();
    let redis_task = wire_out::redis::task::to_dto(&task);

    let _ = cmd("MULTI").query_async::<_, ()>(&mut *conn).await;

    let _ = cmd("ZADD")
        .arg(&[
            "schedules",
            &timestamp_next_execution_time.to_string(),
            //todo: change to id
            &task.name,
        ])
        .query_async::<_, ()>(&mut conn)
        .await;

    let _ = cmd("SET")
        .arg(&[
            format!("task_{}", task.name.to_string()),
            serde_json::to_string(&redis_task).unwrap(),
        ])
        .query_async::<_, ()>(&mut conn)
        .await;

    let exec_results = cmd("EXEC").query_async::<_, ()>(&mut *conn).await;

    return match exec_results {
        Err(_) => Err(format!("It wasn't possible to insert on Redis")),
        Ok(_) => Ok(()),
    };
}
