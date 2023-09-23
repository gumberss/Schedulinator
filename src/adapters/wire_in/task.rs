use crate::schemas::{models, wire_in};

pub fn to_model(dto: &wire_in::task::Task) -> models::task::Task {
    models::task::Task {
        execution_timeout: dto.execution_timeout,
        name: dto.name.clone(),
        schedule: dto.schedule.clone(),
        retry_policy: models::task::RetryPolicy {
            interval: dto.retry_policy.interval,
            jitter_limit: dto.retry_policy.jitter_limit,
            times: dto.retry_policy.times,
        },
    }
}
