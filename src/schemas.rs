pub mod components;
pub mod models {
    pub mod task;
}

pub mod wire_in {
    pub mod task;
}

pub mod wire_out {
    pub mod db {
        pub mod task;
    }
    pub mod redis {
        pub mod task;
    }
}
