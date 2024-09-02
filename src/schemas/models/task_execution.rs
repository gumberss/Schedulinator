use crate::schemas::models;

#[derive(Debug)]
pub struct TaskExecutionData {
    pub name: String,
    pub score: i64,
    pub task: models::task::Task,
}

impl Clone for TaskExecutionData {
    fn clone(&self) -> Self {
        TaskExecutionData {
            task: self.task.clone(),
            name: self.name.clone(),
            score: self.score.clone(),
        }
    }
}
