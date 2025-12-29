use uuid::Uuid;
use deadpool_postgres::{Object};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum PostServiceError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),
    
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub async fn create<'a>(client: Object, user_id: Uuid, text: &String) -> Result<i64, PostServiceError> {
    let res = client.query_one(
        "INSERT INTO posts (user_id, text) VALUES ($1, $2) RETURNING id", 
        &[&user_id, text]
    ).await?;
    let id: i64 = res.get(0);    
    Ok(id)
}