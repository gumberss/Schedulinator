use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]

pub struct Task {
    pub n: String,
    pub s: String,
    pub rp: RetryPolicy,
    pub et: i32,
}
#[derive(Deserialize, Serialize)]
pub struct RetryPolicy {
    pub t: i32,
    pub i: i32,
    pub j: i32,
}
