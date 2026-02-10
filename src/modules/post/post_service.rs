use uuid::Uuid;
use thiserror::Error;
use crate::modules::post::{model::Post, post_cache::{PostCache, MockPostCache}, repository::{PostRepository, MockPostRepository, PostRepositoryError}};
use crate::modules::friend::{repository::{FriendRepository, MockFriendRepository}};
use log::{error, warn};
use crate::modules::ext::extensions::ResultExt;
use async_trait::async_trait; 
use chrono::{DateTime, Utc};

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

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use uuid::Uuid;
    use crate::modules::post::{model::Post, post_cache::MockPostCache, repository::MockPostRepository};
    use crate::modules::friend::{repository::MockFriendRepository};    
    use chrono::Utc;

    #[tokio::test]
    async fn test_create_post_success() {
        let mut mock_repo = MockPostRepository::new();
        let mut mock_friends = MockFriendRepository::new();
        let mut mock_cache = MockPostCache::new();
        let u_id = Uuid::new_v4(); 
        let post_id = Uuid::new_v4();       
        mock_repo.expect_create()
            .with(eq(u_id), eq("Hello world".to_string()))
            .times(1)
            .returning(move |_, _| Ok(Post { id: post_id.clone(), text: "Hello world".to_string(), author_user_id: Uuid::new_v4(), timestamp: Utc::now()}));
        mock_friends.expect_get_followers_ids()
            .with(eq(u_id))
            .times(1)
            .returning(|_| Ok(vec![Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap()]));

        mock_cache.expect_save_post().returning(|_| Ok(0));
        mock_cache.expect_update_followers_feeds().returning(|_, _| Ok(0));        

        let result = PostServiceImpl::new(mock_repo, mock_friends, mock_cache).create(u_id, &"Hello world".to_string()).await;
        assert!(result.is_ok());
        let id = result.ok().unwrap();
        assert_eq!(id, post_id);
    }
}

