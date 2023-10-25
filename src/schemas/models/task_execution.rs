use crate::schemas::models;

#[derive(Debug)]
pub struct TaskExecutionData {
    pub name: String,
    pub score: i64,
    pub task: models::task::Task,
}
