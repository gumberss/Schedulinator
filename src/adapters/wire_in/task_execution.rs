use crate::schemas::models;
use crate::schemas::models::task_execution::TaskExecutionData;
use crate::schemas::wire_out;
use deadpool_redis::redis::Value;

pub fn bulk_cache_to_model(items: Vec<Value>) -> Vec<TaskExecutionData> {
    items
        .iter()
        .filter_map(|item| {
            if let Value::Bulk(bulk_data) = item {
                bulk_data.chunks_exact(3).filter_map(cache_to_model).next()
            } else {
                None
            }
        })
        .collect()
}

pub fn cache_to_model(pair: &[Value]) -> Option<TaskExecutionData> {
    match pair {
        [Value::Data(member), Value::Int(score), Value::Data(complete)] => {
            let member_str = String::from_utf8(member.to_vec()).unwrap();
            let cache_task_str = String::from_utf8(complete.to_vec()).unwrap();
            match serde_json::from_str::<wire_out::redis::task::Task>(&cache_task_str) {
                Ok(cache_task) => {
                    let task = from_dto(&cache_task);
                    let score_i64 = *score;
                    Some(TaskExecutionData {
                        name: member_str,
                        score: score_i64,
                        task,
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn from_dto(task: &wire_out::redis::task::Task) -> models::task::Task {
    models::task::Task {
        execution_timeout: task.et,
        name: task.n.to_owned(),
        url: task.u.to_owned(),
        schedule: task.s.to_owned(),
        retry_policy: models::task::RetryPolicy {
            interval: task.rp.i,
            jitter_limit: task.rp.j,
            times: task.rp.t,
        },
    }
}
