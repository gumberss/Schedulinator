pub struct Task {
    pub name: String,
    pub schedule: String,
    pub url: String,
    pub retry_policy: RetryPolicy,
    pub execution_timeout: i32,
}

pub struct RetryPolicy {
    pub times: i32,
    pub interval: i32,
    pub jitter_limit: i32,
}
