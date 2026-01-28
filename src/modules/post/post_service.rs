use uuid::Uuid;
use deadpool_postgres::{Object};
use thiserror::Error;
use crate::modules::post::{repository::{PostRepositoryError, PostRepositoryImpl, PostRepository}, model::Post};
use crate::modules::friend::{repository::{FriendRepositoryImpl, FriendRepository}};
use std::sync::Arc;
use log::{error, warn};
use deadpool_redis::{redis::{cmd}};
use crate::modules::utils::cache;

const DEFAULT_FEED_SIZE: i64 = 100;

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
    friends_repository: FriendRepositoryImpl,
    redis_pool: &'a deadpool_redis::Pool,
} 

impl <'a> PostServiceImpl<'a> {    
    pub fn new(client: Object, redis_pool: &'a deadpool_redis::Pool) -> PostServiceImpl<'a> {
        let client = Arc::new(client);
        PostServiceImpl { 
            repository: PostRepositoryImpl::new(Arc::clone(&client)), 
            friends_repository: FriendRepositoryImpl::new(Arc::clone(&client)), 
            redis_pool 
        }
    }

    async fn fetch_from_db(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
        let feed = self.repository.feed(user_id, limit, offset).await?;        
        Ok(feed)
    }    

    async fn get_post_ids_from_cache(&self, conn: &mut deadpool_redis::Connection, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<String>, ()> {
        let cache_key = format!("highload/post/feed/ids/{}", user_id);  
        match cmd("ZREVRANGE")
            .arg(&cache_key)
            .arg(offset.unwrap_or(0)) 
            .arg(limit.unwrap_or(DEFAULT_FEED_SIZE))
            .query_async(conn)
            .await {
            Ok(result) => Ok(result),
            Err(e) => {
                warn!("Failed to get ids from cache {}", e);
                Err(())
            }
        }
    }

    async fn check_post_ids_in_cache(&self, conn: &mut deadpool_redis::Connection, user_id: Uuid) -> bool {
        let feed_existence_marker_key = format!("highload/post/feed/exists/{}", user_id);          
        let exists: Result<u8, _> = cmd("EXISTS")
            .arg(&feed_existence_marker_key)
            .query_async(conn).await; 
        match exists {
            Ok(exists) => exists > 0,
            Err(e) => {
                warn!("Failed to check ids in cache");
                false
            }
        }
    }

    async fn save_ids_to_cache(&self, conn: &mut deadpool_redis::Connection, user_id: Uuid, posts: &Vec<Post>) {
        let cache_key = format!("highload/post/feed/ids/{}", user_id);  
        let mut args = Vec::new();
        for post in posts {
            args.push(post.timestamp.to_string()); 
            args.push(post.id.to_string());            
        }
        if args.is_empty() {
            return;
        }    
        if let Err(e) = cmd("ZADD")
            .arg(&cache_key)
            .arg(&args)                                
            .query_async::<()>(conn)
            .await {
            warn!("Failed to save user's posts ids to cache {}", e);                
        }    
    }

    async fn mark_cache_has_ids(&self, conn: &mut deadpool_redis::Connection, user_id: Uuid) {
        let feed_existence_marker_key = format!("highload/post/feed/exists/{}", user_id);
        if let Err(e) = cmd("SET")
            .arg(&feed_existence_marker_key)
            .arg("1")
            .arg("EX")
            .arg(3600)
            .query_async::<()>(conn)
            .await {
            warn!("Redis caching error {}", e);
        }  
    }

    async fn get_instances_by_ids(&self, conn: &mut deadpool_redis::Connection, ids: &Vec<String>) -> Result<Vec<Post>, ()> {
        if ids.is_empty() {
            return Ok(vec!())
        } 
        let posts_data_store = "highload/post/post_by_id".to_string();
        let result: Result<Vec<Option<String>>, _> = cmd("HMGET")
            .arg(&posts_data_store)
            .arg(&ids)
            .query_async( conn)
            .await;
        match result {
            Ok(json_posts) => {
                let posts: Vec<Post> = json_posts
                    .into_iter()
                    .flatten() 
                    .filter_map(|s| serde_json::from_str(&s).ok())
                    .collect();
                Ok(posts)
            },
            Err(e) => {
                warn!("Failed to load posts by ids {}", e);
                Err(())
            }
        }                        
    }
}

impl <'b> PostService for PostServiceImpl<'b> {    

    async fn create<'a>(&self, user_id: Uuid, text: &String) -> Result<Uuid, PostServiceError> {        
        let insert = self.repository.create(user_id, text).await;
        let followers = self.friends_repository.get_followers_ids(user_id).await;
        match insert {
            Ok(post) => {
                match self.redis_pool.get().await {
                    Ok(mut conn) => {
                        let item_key = format!("post:{}", uuid);
                        let _ = cmd("HSET")
                            .arg("posts_data_store")
                            .arg(&item_key)
                            .arg(serde_json::to_string(&item_key).unwrap())
                            .query_async::<()>(&mut conn)
                            .await;
                        match followers {
                            Ok(followers) => {
                                for follower_id in &followers {
                                    let cache_key = format!("highload/post/feed/ids/{}", follower_id);
                                    cmd("ZADD")
                                        .arg(&cache_key)
                                        .arg(post.timestamp) // Score для сортировки
                                        .arg(post.to_string())        
                                        .query_async::<()>(&mut conn)
                                        .await
                                        .ok();
                                }
                            },
                            Err(e) => {
                                warn!("Failed to get user's followers {}: {}", user_id, e);
                            }
                        }
                    },                    
                    Err(e) => {
                        warn!("Redis pool connection error {}", e);
                    }
                }
                Ok(uuid)
            },
            Err(e) => Err(PostServiceError::Database(e))
        }
    }

    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<(), PostServiceError> {
        let result = self.repository.update(user_id, post_id, text).await;
        match result {
            Ok(_) => {
                match self.redis_pool.get().await {
                    Ok(mut conn) => {
                        let item_key = format!("post:{}", post_id);
                        let _ = cmd("HSET")
                            .arg("posts_data_store")
                            .arg(&item_key)
                            .arg(serde_json::to_string(&item_key).unwrap())
                            .query_async::<()>(&mut conn)
                            .await;
                    },                    
                    Err(e) => {
                        warn!("Redis pool connection error {}", e);
                    }
                }
                Ok(())
            },
            Err(e) => Err(PostServiceError::Database(e))
        }
    }

    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError> {
        let result = self.repository.delete(user_id, post_id).await;
        match result {
            Ok(_) => {
                match self.redis_pool.get().await {
                    Ok(mut conn) => {
                        cmd("HDEL")
                            .arg("posts_data_store")
                            .arg(format!("post:{}", post_id))
                            .query_async::<()>(&mut conn)
                            .await
                            .ok();                        
                    },
                    Err(e) => {
                        warn!("Redis pool connection error {}", e);                                            
                    }                    
                };
                Ok(())    
            },
            Err(e) => Err(PostServiceError::Database(e))
        }
    }

    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError> {   
        let posts_data_store = "highload/post/post_by_id".to_string();  
        cache::hget_or_set_cache(
            self.redis_pool,
            &posts_data_store, 
            || async {
                match self.repository.get(post_id).await {
                    Ok(v) => Ok(v),
                    Err(e) => Err(PostServiceError::Database(e))
                }
            },
            &format!("post:{}", post_id).to_string()
        ).await
    }   

    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
        let mut conn = match self.redis_pool.get().await {
            Ok(c) => c,
            Err(_) => {
                warn!("Redis pool error, fetching directly from DB");
                return self.fetch_from_db(user_id, limit, offset).await;
            }
        };
        if self.check_post_ids_in_cache(&mut conn, user_id).await {
            if let Ok(ids) = self.get_post_ids_from_cache(&mut conn, user_id, limit, offset).await {          
                if let Ok(posts) = self.get_instances_by_ids(&mut conn, &ids).await {
                    return Ok(posts);
                }
            }
        }       
        let posts = self.fetch_from_db(user_id, limit, offset).await?;
        self.save_ids_to_cache(&mut conn, user_id, &posts).await;
        self.mark_cache_has_ids(&mut conn, user_id).await;
        Ok(posts)      
    }
}

