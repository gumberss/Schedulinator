use deadpool_postgres::Pool as PostgressPool;
use deadpool_redis::Pool;

pub struct AppComponents {
    pub app_name: String,
    pub redis_pool: Pool,
    pub postgress_pool: PostgressPool,
}

impl Clone for AppComponents {
    fn clone(&self) -> Self {
        AppComponents {
            app_name: self.app_name.clone(),
            redis_pool: self.redis_pool.clone(),
            postgress_pool: self.postgress_pool.clone(),
        }
    }
}
