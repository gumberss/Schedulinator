pub struct Task {
    pub id: i64,
    pub name: String,
    pub schedule: String,
    pub execution_timeout: i32,
    pub retry_times: i32,
    pub retry_interval: i32,
    pub retry_jitter_limit: i32,
}
