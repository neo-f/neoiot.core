mod redis_cache;
use crate::errors::Result;
use poem::async_trait;
pub use redis_cache::RedisCache;

#[async_trait]
pub trait Cache: Send + Sync + 'static {
    async fn block_pop(&self, key: &str, timeout: usize) -> Result<String>;
    async fn lpush(&self, key: &str, value: &str) -> Result<()>;
}
