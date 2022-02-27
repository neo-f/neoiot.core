use crate::errors::Result;
use poem::async_trait;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct RedisCache {
    pub conn: redis::aio::MultiplexedConnection,
}

impl RedisCache {
    pub async fn new(redis_dsn: impl Into<String>) -> Self {
        let client = redis::Client::open(redis_dsn.into()).unwrap();
        let conn = client.get_multiplexed_async_connection().await.unwrap();
        Self { conn }
    }
}

#[async_trait]
impl super::Cache for RedisCache {
    async fn block_pop(&self, key: &str, timeout: usize) -> Result<String> {
        Ok(self.conn.clone().blpop(key, timeout).await?)
    }

    async fn lpush(&self, key: &str, value: &str) -> Result<()> {
        self.conn.clone().rpush(key, value).await?;
        Ok(())
    }
}
