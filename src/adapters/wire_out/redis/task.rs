use crate::schemas::{models, wire_out};

pub fn to_dto(task: &models::task::Task) -> wire_out::redis::task::Task {
    wire_out::redis::task::Task {
        et: task.execution_timeout,
        n: task.name.clone(),
        s: task.schedule.clone(),
        rp: wire_out::redis::task::RetryPolicy {
            i: task.retry_policy.interval,
            j: task.retry_policy.jitter_limit,
            t: task.retry_policy.times,
        },
    }
}
