use crate::schemas::{models, wire_out};

pub fn to_wire(model: &models::task::Task) -> wire_out::http::task::Task {
    wire_out::http::task::Task {
        execution_timeout: model.execution_timeout,
        name: model.name.to_owned(),
        url: model.url.to_owned(),
        schedule: model.schedule.to_owned(),
        retry_policy: wire_out::http::task::RetryPolicy {
            interval: model.retry_policy.interval,
            jitter_limit: model.retry_policy.jitter_limit,
            times: model.retry_policy.times,
        },
    }
}
