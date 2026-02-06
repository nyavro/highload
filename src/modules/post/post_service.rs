use uuid::Uuid;
use thiserror::Error;
use crate::modules::post::{model::Post, post_cache::{PostCache}, repository::{PostRepository, PostRepositoryError}};
use crate::modules::friend::{repository::{FriendRepository}};
use log::{error, warn};
use crate::modules::ext::extensions::ResultExt;
use async_trait::async_trait; 

#[derive(Error, Debug)]
pub enum PostServiceError {
    #[error("Database error: {0}")]
    Database(#[from] PostRepositoryError)
}

#[async_trait]
pub trait PostService {
    async fn create(&self, user_id: Uuid, text: &String) -> Result<Uuid, PostServiceError>;
    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<(), PostServiceError>;
    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError>;
    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError>;
    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError>;
}

pub struct PostServiceImpl<R, F, P> 
where
    R: PostRepository,
    F: FriendRepository,
    P: PostCache {
    repository: R,
    friends_repository: F,
    post_cache: P,
} 

impl <R, F, P> PostServiceImpl<R, F, P>
where 
    R: PostRepository,
    F: FriendRepository,
    P: PostCache {    
    pub fn new<'a>(repository: R, friends_repository: F, post_cache: P) -> Self {
        PostServiceImpl { 
            repository, 
            friends_repository,
            post_cache
        }
    }

    async fn fetch_from_db(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
        let feed = self.repository.feed(user_id, limit, offset).await?;        
        Ok(feed)
    }            
}

#[async_trait]
impl <R, F, P> PostService for PostServiceImpl<R, F, P> 
where
    R: PostRepository + Send + Sync,
    F: FriendRepository + Send + Sync,
    P: PostCache + Send + Sync {    
    async fn create(&self, user_id: Uuid, text: &String) -> Result<Uuid, PostServiceError> {        
        let insert = self.repository.create(user_id, text).await;
        let followers = self.friends_repository.get_followers_ids(user_id).await;        
        match insert {
            Ok(post) => {
                if let Err(e) = self.post_cache.save_post(&post).await {
                    warn!("Saving post to cache failed: {}", e);
                }                        
                match followers {
                    Ok(followers) => {
                        self.post_cache.update_followers_feeds(followers, &post).await.warn("Updating followers feeds pipeline failed".to_string());
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
                self.post_cache.save_post(&post).await.warn("Saving post to cache failed".to_string());                
                Ok(())
            },
            Err(e) => Err(PostServiceError::Database(e))
        }
    }

    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError> {
        let result = self.repository.delete(user_id, post_id).await;
        match result {
            Ok(_) => {
                self.post_cache.delete_post(&post_id).await.warn("Deleting post from cache failed".to_string());
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
        self.post_cache.save_post(&item).await.warn("Failed to save to Redis (HSET)".to_string());        
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

#[cfg(test)] // Компилировать этот блок только при запуске тестов
mod tests {
    use super::*; // Импортируем всё из внешнего модуля

    #[test] // Пометка, что это тестовая функция
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}

