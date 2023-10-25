pub mod wire_in {
    pub mod task;
    pub mod task_execution;
}

pub mod wire_out {
    pub mod redis {
        pub mod task;
    }
    pub mod http {
        pub mod task;
    }
}
