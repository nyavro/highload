use uuid::Uuid;
use deadpool_postgres::{Object};
use thiserror::Error;
use log::{info, error};
use crate::modules::post::{repository::{PostRepositoryError, PostRepositoryImpl, PostRepository}, model::Post};
use deadpool_redis::{redis::{cmd, FromRedisValue}, Config, Runtime};
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum PostServiceError {
    #[error("Database error: {0}")]
    Database(#[from] PostRepositoryError)
}

pub trait PostService {
    async fn create<'a>(&self, user_id: Uuid, text: &String) -> Result<Uuid, PostServiceError>;
    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<(), PostServiceError>;
    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError>;
    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError>;
    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError>;
}

pub struct PostServiceImpl<'a> {
    repository: PostRepositoryImpl,
    redis_pool: &'a deadpool_redis::Pool,
}

impl <'a> PostServiceImpl<'a> {
    pub fn new(client: Object, redis_pool: &deadpool_redis::Pool) -> PostServiceImpl {

        PostServiceImpl { repository: PostRepositoryImpl::new(client), redis_pool }
    }

    async fn fetch_from_db(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
        let feed = self.repository.feed(user_id, limit, offset).await?;        
        Ok(feed)
    }

    async fn fetch_from_db_then_cache(&self, mut conn: deadpool_redis::Connection, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
        let feed = self.repository.feed(user_id, limit, offset).await?;        
        Ok(feed)
    }
}

impl <'b> PostService for PostServiceImpl<'b> {    

    async fn create<'a>(&self, user_id: Uuid, text: &String) -> Result<Uuid, PostServiceError> {        
        let uuid = self.repository.create(user_id, text).await?;
        Ok(uuid)
    }

    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<(), PostServiceError> {
        let result = self.repository.update(user_id, post_id, text).await?;
        Ok(result)
    }

    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError> {
        let result = self.repository.delete(user_id, post_id).await?;
        Ok(result)
    }

    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError> {
        let post = self.repository.get(post_id).await?;
        Ok(post)
    }    

    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {        
        match self.redis_pool.get().await {
            Ok(mut conn) => {
                let value: Option<String> = cmd("GET")
                    .arg(&["highload/post/feed", &user_id.to_string()])
                    .query_async(&mut conn)
                    .await
                    .unwrap();
                match value {
                    Some(json) => {
                        let posts: Vec<Post> = serde_json::from_str(&json).unwrap();
                        Ok(posts)
                    },
                    None => {
                        self.fetch_from_db_then_cache(conn, user_id, limit, offset).await
                    }
                }                
            },
            Err(err) => {
                error!("Failed to connect to Redis {}", err);
                info!("Fetching feed from db");
                self.fetch_from_db(user_id, limit, offset).await
            }
        }     
    }    
}

