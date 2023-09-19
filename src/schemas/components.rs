use deadpool_postgres::Pool as PostgressPool;
use deadpool_redis::Pool;

pub struct AppComponents {
    pub app_name: String,
    pub redis_pool: Pool,
    pub postgress_pool: PostgressPool,
}
