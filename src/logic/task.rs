use crate::schemas::models::task;

pub fn is_minimum_recurrence_time_valid(task: &task::Task) -> bool {
    let minimum_recurrence_time = worst_case_retry(&task.retry_policy);
    match minimum_recurrence_time {
        Some(min_rec_time) => task.execution_timeout > min_rec_time,
        None => false,
    }
}

pub fn worst_case_retry(retry_policy: &task::RetryPolicy) -> Option<i32> {
    let exponential_backoff: i32 = 2;

    (0..=(retry_policy.times - 1) as u32)
        .map(|x| retry_policy.interval * exponential_backoff.pow(x) + retry_policy.jitter_limit)
        .reduce(|acc, x| acc + x)
}
