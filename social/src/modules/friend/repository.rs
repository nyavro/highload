use uuid::Uuid;
use deadpool_postgres::{Pool};
use thiserror::Error;
use std::sync::Arc;
use async_trait::async_trait; 
use mockall::automock;

#[derive(Error, Debug)]
pub enum FriendRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),
    
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
}

#[automock]
#[async_trait]
pub trait FriendRepository {
    async fn get_followers_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, FriendRepositoryError>;   
}

pub struct FriendRepositoryImpl {
    pool: Arc<Pool>
}

impl FriendRepositoryImpl {
    pub fn new(pool: Arc<Pool>) -> Self {
        FriendRepositoryImpl { pool }
    }
}

#[async_trait]
impl FriendRepository for FriendRepositoryImpl {    

    async fn get_followers_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, FriendRepositoryError> {
        let client = self.pool.get().await?;
        let res = client.query(
            "SELECT friend_id FROM friends WHERE user_id = $1", 
            &[&user_id]
        ).await?;
        Ok(res.iter().map(|row| row.get(0)).collect()) 
    }
}