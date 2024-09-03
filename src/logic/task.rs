use crate::schemas::models::{task, task_execution::TaskExecutionData};
use chrono::{DateTime, Utc};
use core::result::Result;
use cron::Schedule;
use std::{error::Error, str::FromStr};

pub fn exceed_recurrence_time_limit(task: &task::Task) -> bool {
    match worst_case_retry(&task.retry_policy) {
        Some(worst_case_retry_time) => worst_case_retry_time > 30_000,
        None => true,
    }
}

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

pub fn new_score(current_score: i64, task: task::Task) -> i64 {
    let current_score_time = DateTime::from_timestamp(current_score, 0);

    let schedule = Schedule::from_str(&task.schedule);

    //todo: here you sohuld decide if you want the next time execution after the last exeution time
    // or after the current time
    let next_execution_time = schedule
        .unwrap()
        .after(&current_score_time.unwrap())
        .take(1)
        .next();

    return next_execution_time.unwrap().timestamp();
}
