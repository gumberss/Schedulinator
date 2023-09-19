use deadpool_redis::{ConnectionAddr, ConnectionInfo, Pool, RedisConnectionInfo, Runtime};
use std::env;

pub fn configure() -> Pool {
    let mut cfg = deadpool_redis::Config::default();
    let redis_host = env::var("REDIS_HOST").unwrap_or("127.0.0.1".into());
    cfg.connection = Some(ConnectionInfo {
        addr: ConnectionAddr::Tcp(redis_host, 6379),
        redis: RedisConnectionInfo {
            db: 0,
            username: None,
            password: Some("pass".to_string()),
        },
    });
    return cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
}
