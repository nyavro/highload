use uuid::Uuid;
use crate::modules::post::{model::Post, post_cache::PostCache, service_provider::{PostService, PostServiceError}};
use log::{info};
use crate::modules::common::ext::extensions::ResultExt;
use async_trait::async_trait; 

pub struct CachedPostService <C, S>
where    
    C: PostCache,
    S: PostService {
    service: S,
    post_cache: C,
}

impl <C, S> CachedPostService<C, S> 
where 
    C: PostCache + Send + Sync,
    S: PostService + Send + Sync {
    pub fn new(service: S, post_cache: C) -> Self {
        CachedPostService { 
            service,
            post_cache
        }
    }
}

#[async_trait]
impl <C, S> PostService for CachedPostService <C, S>
where
    C: PostCache + Send + Sync,
    S: PostService + Send + Sync {    
    async fn create(&self, user_id: Uuid, text: &String) -> Result<Post, PostServiceError> {                
        let post = self.service.create(user_id, text).await?;
        self.post_cache.save_post(&post).await.warn("Saving post to cache failed".to_string());                                                        
        Ok(post)
    }

    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<Post, PostServiceError> {        
        let post = self.service.update(user_id, post_id, text).await?;
        self.post_cache.save_post(&post).await.warn("Saving post to cache failed".to_string());                
        Ok(post)        
    }

    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError> {
        let _ = self.service.delete(user_id, post_id).await?;
        self.post_cache.delete_post(&post_id).await.warn("Deleting post from cache failed".to_string());
        Ok(())        
    }

    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError> {           
        if let Ok(Some(post)) = self.post_cache.get_post(&post_id).await {
            return Ok(post);
        }
        let post = self.service.get(post_id).await?;
        self.post_cache.save_post(&post).await.warn("Save to Redis (HSET) failed".to_string());        
        Ok(post)        
    }   

    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {        
        if let Ok(exists) = self.post_cache.check_feed_exists(user_id).await && exists {
            info!("Cache hit: {}", user_id);
            if let Ok(ids) = self.post_cache.get_user_feed(user_id, limit, offset).await {
                info!("Ids from cache: {:?}", ids);
                let ids_len = ids.len();
                if let Ok(posts) = self.post_cache.get_posts_by_ids(ids).await && posts.len() == ids_len {
                    return Ok(posts);
                }
            }
        }       
        let posts = self.service.feed(user_id, limit, offset).await?;
        if !posts.is_empty() {
            self.post_cache.save_posts(&posts).await.warn(format!("Failed to save posts {}", user_id));
            self.post_cache.save_user_feed(user_id, &posts).await.warn(format!("Failed to save user's {} feed", user_id));
            self.post_cache.mark_feed_exists(user_id).await.warn(format!("Failed to mark feed exists. Feed {}", user_id));
        }        
        Ok(posts)      
    }
}