use uuid::Uuid;
use deadpool_postgres::{Object};
use thiserror::Error;
use log::{error, warn};
use crate::modules::post::{repository::{PostRepositoryError, PostRepositoryImpl, PostRepository}, model::Post};
use deadpool_redis::{redis::{cmd}};
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

async fn get_or_set_cache<T, E, F, Fut>(redis_pool: &deadpool_redis::Pool, cache_key: &str, fetch_func: F) -> Result<T, E> 
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>, {             
        if let Ok(mut conn) = redis_pool.get().await {
            let cached_value: Option<String> = cmd("GET")
                .arg(&cache_key)
                .query_async(&mut conn)
                .await
                .ok();
            if let Some(json) = cached_value {
                match serde_json::from_str(&json) {
                    Ok(res) => return Ok(res),
                    Err(e) => {
                        error!("Failed to deserialize cache for {}: {}", cache_key, e);
                    }
                }
            }
            let data = fetch_func().await?;
            if let Ok(json) = serde_json::to_string(&data) {
                if let Err(e) = cmd("SET")
                    .arg(cache_key)
                    .arg(json)
                    .arg("EX")
                    .arg(3600)
                    .query_async::<()>(&mut conn)
                    .await {
                    warn!("Redis caching error {}", e);
                }                    
            }
            Ok(data)                            
        } else {
            warn!("Redis pool error, fetching directly from DB");
            fetch_func().await    
        }        
    }

impl <'a> PostServiceImpl<'a> {
    pub fn new(client: Object, redis_pool: &deadpool_redis::Pool) -> PostServiceImpl {
        PostServiceImpl { repository: PostRepositoryImpl::new(client), redis_pool }
    }

    async fn fetch_from_db(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
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
        let cache_key = format!("highload/post/feed/{}", user_id);     
        get_or_set_cache(
            self.redis_pool,
            &cache_key, 
            || async {
                self.fetch_from_db(user_id, limit, offset).await
            }
        ).await
    }        
}

