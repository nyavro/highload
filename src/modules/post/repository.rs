use openapi::apis::user;
use uuid::Uuid;
use deadpool_postgres::{Object};
use thiserror::Error;
use crate::modules::post::model::Post;
use std::sync::Arc;

#[derive(Error, Debug)]
pub enum PostRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),
    
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub trait PostRepository {
    async fn create<'a>(&self, user_id: Uuid, text: &String) -> Result<Post, PostRepositoryError>;
    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<Post, PostRepositoryError>;
    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostRepositoryError>;
    async fn get(&self, post_id: Uuid) -> Result<Post, PostRepositoryError>;
    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostRepositoryError>;    
}

pub struct PostRepositoryImpl {
    client: Arc<Object>
}

impl PostRepositoryImpl {
    pub fn new(client: Arc<Object>) -> Self {
        PostRepositoryImpl { client }
    }
}

impl PostRepository for PostRepositoryImpl {    

    async fn create<'a>(&self, user_id: Uuid, text: &String) -> Result<Post, PostRepositoryError> {
        let res = self.client.query_one(
            "INSERT INTO posts (user_id, text) VALUES ($1, $2) RETURNING id, created_at", 
            &[&user_id, text]
        ).await?;
        let id: Uuid = res.get("id");
        let timestamp: chrono::DateTime<chrono::Utc> = res.get("created_at");                     
        Ok(Post {id, text: text.to_string(), author_user_id: user_id, timestamp})
    }

    async fn update(&self, user_id: Uuid, id: Uuid, text: &String) -> Result<Post, PostRepositoryError> {
        let res = self.client.query_one(
            "UPDATE posts SET text=$1,updated_at=NOW() WHERE user_id=$2 AND id=$3 RETURNING updated_at", 
            &[text, &user_id, &id]
        ).await?;    
        let timestamp: chrono::DateTime<chrono::Utc> = res.get("updated_at");                     
        Ok(Post {id, text: text.to_string(), author_user_id: user_id, timestamp})        
    }

    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostRepositoryError> {
        let rows_affected = self.client.execute(
            "DELETE FROM posts WHERE user_id=$1 AND id=$2", 
            &[&user_id, &post_id]
        ).await?;    
        if rows_affected > 0 {
            Ok(())
        } else {
            Err(PostRepositoryError::Internal("Not updated".to_string()))
        }
    }

    async fn get(&self, post_id: Uuid) -> Result<Post, PostRepositoryError> {
        let res = self.client.query_one(
            "SELECT text,user_id, updated_at FROM posts WHERE id=$1", 
            &[&post_id]
        ).await?;    
        let text: String = res.get("text");    
        let author_user_id: Uuid = res.get("user_id");
        let timestamp: chrono::DateTime<chrono::Utc> = res.get("updated_at");    
        Ok(Post {id: post_id, text, author_user_id, timestamp})
    }

    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostRepositoryError> {
        let res = self.client.query(
            "SELECT p.text,p.user_id,p.id, p.updated_at 
                FROM (SELECT friend_id AS f_id FROM friends WHERE user_id=$1) q 
                JOIN posts p ON q.f_id = p.user_id ORDER BY p.created_at DESC LIMIT $2 OFFSET $3", 
            &[&user_id, &limit, &offset]
        ).await?;        
        Ok(
            res.iter().map(|row| {
                let text: String = row.get(0);    
                let author_user_id: Uuid = row.get(1);    
                let id: Uuid = row.get(2);
                let timestamp: chrono::DateTime<chrono::Utc> = row.get("updated_at");
                Post {id, text, author_user_id, timestamp}
            }).collect()
        )    
    }
}