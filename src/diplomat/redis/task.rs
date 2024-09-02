use deadpool_redis::{redis::cmd, Pool};

use crate::adapters::wire_out;
use crate::schemas::models::task;

pub async fn insert(
    timestamp_next_execution_time: i64,
    task: &task::Task,
    pool: Pool,
) -> Result<(), String> {
    let conn_future = pool.get();
    let redis_task = wire_out::redis::task::to_dto(task);
    let mut conn = conn_future.await.unwrap();

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
            format!("task_{}", task.name),
            serde_json::to_string(&redis_task).unwrap(),
        ])
        .query_async::<_, ()>(&mut conn)
        .await;

    let exec_results = cmd("EXEC").query_async::<_, ()>(&mut *conn).await;

    match exec_results {
        Err(_e) => Err("It wasn't possible to insert on Redis".to_string()),
        Ok(_) => Ok(()),
    }
}
