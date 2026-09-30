use std::sync::Arc;

use async_trait::async_trait;
use fred::clients::Pool;
use fred::prelude::*;
use thiserror::Error;

#[derive( Error, Debug)]
pub enum CounterError {
    #[error("Redis error: {0}")]
    Redis(#[from] fred::error::Error),
    #[error("Counter error: {0}")]
    Other(String)
}

#[derive(Debug, Clone)]
pub struct CounterRepositoryImpl {
    pool: Arc<Pool>
}

impl CounterRepositoryImpl {
    pub fn new(pool: Arc<Pool>) -> Self {
        Self { pool }
    }

    fn make_key(user_id: &str) -> String {
        format!("unread:{}", user_id)
    }
}

#[async_trait]
pub trait CounterRepository {
    async fn get(&self, user_id: &str) -> Result<i64, CounterError>;
    async fn increment(&self, user_id: &str) -> Result<i64, CounterError>;
    async fn decrement(&self, user_id: &str) -> Result<i64, CounterError>;
}

#[async_trait]
impl CounterRepository for CounterRepositoryImpl {
    async fn increment(&self, user_id: &str) -> Result<i64, CounterError> {
        let key = Self::make_key(user_id);
        let client = self.pool.next();
        let val: i64 = client.incr(&key).await?;
        Ok(val)
    }

    async fn decrement(&self, user_id: &str) -> Result<i64, CounterError> {
        let key = Self::make_key(user_id);
        let client = self.pool.next();
        let val: i64 = client.decr(&key).await?;
        Ok(val)
    }

    async fn get(&self, user_id: &str) -> Result<i64, CounterError> {
        let key = Self::make_key(user_id);
        let client = self.pool.next();
        let val: i64 = client.get(&key).await?;
        Ok(val)
    }
}