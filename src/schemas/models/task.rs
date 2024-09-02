#[derive(Debug)]
pub struct Task {
    pub name: String,
    pub schedule: String,
    pub url: String,
    pub retry_policy: RetryPolicy,
    pub execution_timeout: i32,
}

impl Clone for Task {
    fn clone(&self) -> Self {
        Task {
            name: self.name.clone(),
            schedule: self.schedule.clone(),
            url: self.url.clone(),
            retry_policy: self.retry_policy.clone(),
            execution_timeout: self.execution_timeout.clone(),
        }
    }
}

#[derive(Debug)]
pub struct RetryPolicy {
    pub times: i32,
    pub interval: i32,
    pub jitter_limit: i32,
}

impl Clone for RetryPolicy {
    fn clone(&self) -> Self {
        RetryPolicy {
            times: self.times.clone(),
            interval: self.interval.clone(),
            jitter_limit: self.jitter_limit.clone(),
        }
    }
}
