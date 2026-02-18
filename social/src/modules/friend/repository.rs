use uuid::Uuid;
use deadpool_postgres::{Object};
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

    #[error("Internal error: {0}")]
    Internal(String),
}

#[automock]
#[async_trait]
pub trait FriendRepository {
    async fn get_followers_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, FriendRepositoryError>;   
}

pub struct FriendRepositoryImpl {
    client: Arc<Object>
}

impl FriendRepositoryImpl {
    pub fn new(client: Arc<Object>) -> Self {
        FriendRepositoryImpl { client }
    }
}

#[async_trait]
impl FriendRepository for FriendRepositoryImpl {    

    async fn get_followers_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, FriendRepositoryError> {
        let res = self.client.query(
            "SELECT friend_id FROM friends WHERE user_id = $1", 
            &[&user_id]
        ).await?;
        Ok(res.iter().map(|row| row.get(0)).collect()) 
    }
}