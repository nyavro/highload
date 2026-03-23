use async_trait::async_trait;
use uuid::Uuid;
use crate::modules::{common::ext::extensions::ResultExt, post::{followers::follower_event_bus::FollowerEventListener, model::Post, post_cache::FeedCache}};


pub struct CachingPostListener<C> 
where 
    C: FeedCache,{
    pub cache: C,
}

impl <C> CachingPostListener<C> 
where 
    C: FeedCache, {
    pub fn new(cache: C) -> Self {
        CachingPostListener {             
            cache
        }
    }
}

#[async_trait]
impl <C> FollowerEventListener for CachingPostListener<C> 
where 
    C: FeedCache + Send + Sync, {
    
    async fn create(&self, _: &Uuid, followers: &Vec<Uuid>, post: &Post) {
        self.cache.process_save(followers, &post).await.warn("Failed to notify followers feeds on post create".to_string());
    }

    async fn update(&self, _: &Uuid, followers: &Vec<Uuid>, post: &Post) {
        self.cache.process_save(followers, &post).await.warn("Failed to notify followers feeds on post update".to_string());
    } 

    async fn delete(&self, _: &Uuid, followers: &Vec<Uuid>, post_id: &Uuid) {
        self.cache.process_delete(followers, post_id).await.warn("Failed to notify followers feeds on post create".to_string());
    }
}