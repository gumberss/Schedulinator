use crate::diplomat::redis::task;
use crate::logic;
use crate::schemas::components::AppComponents;
use crate::schemas::models::task_execution::TaskExecutionData;
use deadpool_redis::redis::cmd;
use deadpool_redis::redis::RedisError;
use deadpool_redis::redis::Value;
use deadpool_redis::Pool;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use reqwest::Client;
use tokio::time::{sleep, Duration};

pub async fn execute_tasks(components: AppComponents) {
    loop {
        let redis_pool = components.redis_pool.clone();

        let tasks_datas_response = task::get_tasks_to_be_processed(&redis_pool).await;

        let client = reqwest::Client::new();

        let updated_keys = match tasks_datas_response {
            Ok(tasks_datas) => {
                let mut tasks = tasks_datas
                    .into_iter()
                    .map(|execution_data: TaskExecutionData| {
                        let client = client.clone();
                        let redis_pool2 = components.redis_pool.clone();
                        async move { process(execution_data, client.clone(), &redis_pool2).await }
                    })
                    .collect::<FuturesUnordered<_>>();

                let mut successes = Vec::new();

                while let Some(result) = tasks.next().await {
                    match result {
                        Ok(data) => successes.push(data),
                        Err(e) => println!("{}", e),
                    }
                }

                Ok(successes)
            }
            Err(e) => Err(e),
            _ => Err("Unexpected error".to_string()),
        };

        dbg!(&updated_keys);

        sleep(Duration::from_millis(100)).await;
    }
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
