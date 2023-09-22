use deadpool_redis::{
    ConnectionAddr, ConnectionInfo, Pool, PoolConfig, RedisConnectionInfo, Runtime, Timeouts,
};
use std::env;
use std::time::Duration;

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

    cfg.pool = Some(PoolConfig {
        max_size: 400,
        timeouts: Timeouts {
            wait: Some(Duration::from_secs(30)),
            create: Some(Duration::from_secs(30)),
            recycle: Some(Duration::from_secs(30)),
        },
    });

    return cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
}
