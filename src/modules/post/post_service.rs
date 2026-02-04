use openapi::apis::user;
use uuid::Uuid;
use deadpool_postgres::{Object};
use thiserror::Error;
use crate::modules::post::{model::Post, post_cache::{PostCacheImpl, PostCache}, repository::{PostRepository, PostRepositoryError, PostRepositoryImpl}};
use crate::modules::friend::{repository::{FriendRepositoryImpl, FriendRepository}};
use std::sync::Arc;
use log::{error, warn};
use crate::modules::ext::extensions::ResultExt;

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
    post_cache: PostCacheImpl<'a>,
} 

impl <'a> PostServiceImpl<'a> {    
    pub fn new(client: Object, redis: &'a fred::prelude::Pool) -> Self {
        let client = Arc::new(client);
        PostServiceImpl { 
            repository: PostRepositoryImpl::new(Arc::clone(&client)), 
            friends_repository: FriendRepositoryImpl::new(Arc::clone(&client)),             
            post_cache: PostCacheImpl::new(redis)
        }
    }

    async fn fetch_from_db(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
        let feed = self.repository.feed(user_id, limit, offset).await?;        
        Ok(feed)
    }            
}

impl <'b> PostService for PostServiceImpl<'b> {    

    async fn create<'a>(&self, user_id: Uuid, text: &String) -> Result<Uuid, PostServiceError> {        
        let insert = self.repository.create(user_id, text).await;
        let followers = self.friends_repository.get_followers_ids(user_id).await;        
        match insert {
            Ok(post) => {
                if let Err(e) = self.post_cache.save_post(&post).await {
                    warn!("Saving post to cache failed: {}", e);
                }                        
                match followers {
                    Ok(followers) => {
                        if let Err(e) = self.post_cache.update_followers_feeds(followers, &post).await {
                            warn!("Updating followers feeds pipeline failed: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to get user's followers from DB for user {}: {}", user_id, e);
                    }
                }                                
                Ok(post.id)
            },
            Err(e) => Err(PostServiceError::Database(e))
        }
    }

    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<(), PostServiceError> {
        let result = self.repository.update(user_id, post_id, text).await;
        match result {
            Ok(post) => {
                if let Err(e) = self.post_cache.save_post(&post).await {
                    warn!("Saving post to cache failed: {}", e);
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
                if let Err(e) = self.post_cache.delete_post(&post_id).await {
                    warn!("Deleting post from cache failed: {}", e);
                }                
                Ok(())    
            },
            Err(e) => Err(PostServiceError::Database(e))
        }
    }

    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError> {           
        if let Ok(Some(post)) = self.post_cache.get_post(&post_id).await {
            return Ok(post);
        }
        let item = match self.repository.get(post_id).await {
            Ok(v) => v,
            Err(e) => return Err(PostServiceError::Database(e))
        };
        if let Err(e) = self.post_cache.save_post(&item).await {
            warn!("Failed to save to Redis (HSET): {}", e);
        }
        Ok(item)        
    }   

    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {        
        if let Ok(exists) = self.post_cache.check_feed_exists(user_id).await && exists {
            if let Ok(ids) = self.post_cache.get_user_feed(user_id, limit, offset).await {
                if let Ok(posts) = self.post_cache.get_posts_by_ids(ids).await {
                    return Ok(posts);
                }
            }
        }       
        let posts = self.fetch_from_db(user_id, limit, offset).await?;
        if !posts.is_empty() {
            self.post_cache.save_user_feed(user_id, &posts).await.warn(format!("Failed to save user's {} feed", user_id));
            self.post_cache.mark_feed_exists(user_id).await.warn(format!("Failed to mark feed exists. Feed {}", user_id));
        }        
        Ok(posts)      
    }
}

