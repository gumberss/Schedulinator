use crate::diplomat;
use crate::logic::task;
use crate::schemas::models;
use chrono::Utc;
use cron::Schedule;
use deadpool_postgres::Pool as PostgressPool;
use deadpool_redis::Pool;
use std::str::FromStr;

pub async fn insert(
    task: models::task::Task,
    redis_pool: &Pool,
    postgress_pool: &PostgressPool,
) -> Result<models::task::Task, String> {
    let is_valid = task::is_minimum_recurrence_time_valid(&task);

    if !is_valid {
        let retry_worst_case = task::worst_case_retry(&task.retry_policy)
            .unwrap()
            .to_string();
        let timeout = task.execution_timeout;
        return Err(format!("The worst case of the retry policy ({retry_worst_case}) is bigger than the recurrence time {timeout}"));
    }
    let schedule = Schedule::from_str(&task.schedule);

    if let Err(err) = schedule {
        return Err(err.to_string());
    }

    let next_execution_time = schedule.unwrap().upcoming(Utc).take(1).next();

    if next_execution_time.is_none() {
        return Err("There is no next execution for the task".to_owned());
    }
    let timestamp_next_execution_time = next_execution_time.unwrap().timestamp();

    match diplomat::postgres::task::insert(&task, postgress_pool.to_owned()).await {
        Err(err) => return Err(err.as_str().to_owned()),
        Ok(r) => r,
    };

    match diplomat::redis::task::insert(timestamp_next_execution_time, &task, redis_pool.to_owned())
        .await
    {
        Err(_) => Err("It wasn't possible to insert on cache".to_owned()),
        Ok(_) => Ok(task),
    }
}
