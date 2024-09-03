use crate::adapters;
use crate::schemas::models::task_execution::TaskExecutionData;
use deadpool_redis::redis::RedisError;
use deadpool_redis::redis::Value;
use deadpool_redis::Connection;
use deadpool_redis::{redis::cmd, Pool};

use crate::adapters::wire_out;
use crate::schemas::models::task;

pub async fn get_tasks_to_be_processed(
    redis_pool: &Pool,
) -> Result<Vec<TaskExecutionData>, String> {
    let mut conn: Connection = redis_pool.get().await.unwrap();

    let lua_script = r#"
       
    local currentServerTime = redis.call('TIME')
    local thresholdScore = tonumber(currentServerTime[1]) 
    local updatedKeysWithScores = redis.call('ZRANGEBYSCORE', 'schedules', 0, thresholdScore, 'WITHSCORES', 'LIMIT', 0, 5)
    local newScores = {}
    
    for i=1, #updatedKeysWithScores, 2 do
        local member = updatedKeysWithScores[i]
        local score = tonumber(updatedKeysWithScores[i + 1])
        local lock_score = score * -1
        local hidrated = redis.call('GET', 'task_'..member)
        redis.call('ZADD', 'schedules', lock_score, member)
        table.insert(newScores, {member, score, hidrated})
    end
    
    return newScores
    
    "#;

    let keys = ["schedules"];

    let response: Result<Value, RedisError> = cmd("EVAL")
        .arg(lua_script)
        .arg(1)
        .arg(keys.len())
        .arg(&keys)
        .query_async(&mut conn)
        .await;

    let k: Result<Vec<TaskExecutionData>, String> = match response {
        Ok(Value::Bulk(items)) => {
            let tasks = adapters::wire_in::task_execution::bulk_cache_to_model(items);
            Ok(tasks)
        }
        Err(e) => Err(format!("Redis error: {:?}", e)),
        _ => Err("Unexpected response format from Lua script".to_string()),
    };

    return k;
}

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
