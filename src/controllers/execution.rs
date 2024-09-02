use crate::adapters;
use crate::logic;
use crate::schemas::models::task_execution::TaskExecutionData;
use actix_web::Error;
use deadpool_redis::redis::cmd;
use deadpool_redis::redis::RedisError;
use deadpool_redis::redis::Value;
use deadpool_redis::Connection;
use deadpool_redis::Pool;
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use reqwest::Client;
use std::collections;
use tokio::task;

pub async fn execute_tasks(redis_pool: &Pool) -> Result<Vec<TaskExecutionData>, String> {
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

    let client = reqwest::Client::new();

    let updated_keys = match response {
        Ok(Value::Bulk(items)) => {
            dbg!(items.clone());
            let mut tasks = adapters::wire_in::task_execution::bulk_cache_to_model(items)
                .into_iter()
                .map(|execution_data: TaskExecutionData| {
                    let client = client.clone();
                    let local_pool = redis_pool.clone();

                    async move { process(execution_data, client, &local_pool).await }
                })
                .collect::<FuturesUnordered<_>>();

            // Collect results from the spawned tasks
            let mut successes = Vec::new();

            // Process each task as they complete
            while let Some(result) = tasks.next().await {
                match result {
                    Ok(data) => successes.push(data),
                    Err(e) => return Err(e), // Return the first encountered error
                }
            }

            Ok(successes)
        }
        Err(e) => Err(format!("Redis error: {:?}", e)),
        _ => Err("Unexpected response format from Lua script".to_string()),
    };

    dbg!(&updated_keys);

    updated_keys
}

pub async fn process(
    execution_data: TaskExecutionData,
    client: Client,
    redis_pool: &Pool,
) -> Result<TaskExecutionData, String> {
    let response = client
        .clone()
        .post(&execution_data.clone().task.url)
        .send()
        .await;
    match response {
        Ok(response) => {
            if response.status() == reqwest::StatusCode::OK {
                let new_score =
                    logic::task::new_score(execution_data.score, execution_data.clone().task);
                let result: Result<Value, RedisError> = cmd("ZADD")
                    .arg("schedules")
                    .arg(new_score)
                    .arg(&execution_data.name)
                    .query_async(&mut redis_pool.get().await.unwrap())
                    .await;
                match result {
                    Ok(r) => Ok(execution_data.clone()),
                    Err(err) => {
                        dbg!(err);
                        Err("err".to_string())
                    }
                }
            } else {
                return Err("".to_string());
            }
        }
        Err(err) => {
            dbg!(err);
            Err("err".to_string())
        } //todo: retry
    }
}
