use crate::adapters;
use crate::schemas::models::task_execution::TaskExecutionData;
use deadpool_redis::redis::cmd;
use deadpool_redis::redis::RedisError;
use deadpool_redis::redis::Value;
use deadpool_redis::Connection;
use deadpool_redis::Pool;

pub async fn execute_tasks(redis_pool: &Pool) -> Result<Vec<TaskExecutionData>, String> {
    let mut conn: Connection = redis_pool.get().await.unwrap();

    let lua_script = r#"
       
    local currentServerTime = redis.call('TIME')
    local thresholdScore = tonumber(currentServerTime[1]) 
    local updatedKeysWithScores = redis.call('ZRANGEBYSCORE', 'schedules', 0, thresholdScore, 'WITHSCORES', 'LIMIT', 0, 5)
    local newScores = {}
    
    for i=1, #updatedKeysWithScores, 2 do
        local member = updatedKeysWithScores[i]
        local score = tonumber(updatedKeysWithScores[i + 1]) * -1
        local hidrated = redis.call('GET', 'task_'..member)
        redis.call('ZADD', 'schedules', score, member)
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

    let updated_keys: Result<Vec<TaskExecutionData>, String> = match response {
        Ok(Value::Bulk(items)) => Ok(adapters::wire_in::task_execution::bulk_cache_to_model(
            items,
        )),
        Err(e) => Err(format!("Redis error: {:?}", e)),
        _ => Err("Unexpected response format from Lua script".to_string()),
    };
    dbg!(&updated_keys);
    updated_keys
}
