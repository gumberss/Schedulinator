use crate::schemas::{models, wire_out};

pub fn to_dto(task: &models::task::Task) -> wire_out::redis::task::Task {
    wire_out::redis::task::Task {
        et: task.execution_timeout,
        n: task.name.to_owned(),
        u: task.url.to_owned(),
        s: task.schedule.to_owned(),
        rp: wire_out::redis::task::RetryPolicy {
            i: task.retry_policy.interval,
            j: task.retry_policy.jitter_limit,
            t: task.retry_policy.times,
        },
    }
}
