use crate::schemas::models::task;
use deadpool_postgres::{GenericClient, Pool};
use sql_builder::{quote, SqlBuilder};

pub async fn insert(task: &task::Task, pool: Pool) -> Result<(), String> {
    let conn_future = pool.get();

    let mut sql_builder = SqlBuilder::insert_into("tasks");
    sql_builder
        .field("name")
        .field("schedule")
        .field("url")
        .field("executionTimeout")
        .field("retryTimes")
        .field("retryInterval")
        .field("retryJitterLimit");
    sql_builder.values(&[
        &quote(&task.name),
        &quote(&task.schedule),
        &quote(&task.url),
        &quote(&task.execution_timeout),
        &quote(&task.retry_policy.times),
        &quote(&task.retry_policy.interval),
        &quote(&task.retry_policy.jitter_limit),
    ]);

    let this_sql = match sql_builder.sql() {
        Ok(x) => x,
        Err(_) => return Err("Couldn't able to create the query".to_owned()),
    };
    println!("Query: {}", this_sql);

    let mut conn = match conn_future.await {
        Ok(x) => x,
        Err(_) => return Err("Couldn't able to get the connection".to_owned()),
    };
    let transaction = match conn.transaction().await {
        Ok(x) => x,
        Err(_) => return Err("Couldn't able to open the transaction".to_owned()),
    };
    match transaction.batch_execute(&this_sql.as_str()).await {
        Ok(_) => (),
        Err(err) => {
            println!("Error: {}", err.to_string());
            return Err("Couldn't able to execute the query".to_owned());
        }
    };
    match transaction.commit().await {
        Ok(_) => return Ok(()),
        Err(_) => return Err("Couldn't able to commit the query".to_owned()),
    }
}