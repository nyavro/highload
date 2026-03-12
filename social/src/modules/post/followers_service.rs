use uuid::Uuid;
use crate::modules::friend::repository::{FriendRepository, FriendRepositoryError};
use crate::modules::post::model::Post;
use crate::modules::post::service_provider::{PostService, PostServiceError};
use thiserror::Error;
use log::{error, warn};
use async_trait::async_trait;
use std::sync::Arc; 

#[derive(Error, Debug)]
pub enum FollowersServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] FriendRepositoryError)
}

#[async_trait]
pub trait PostListener {
    async fn create(&self, user_id: &Uuid, followers: &Vec<Uuid>, post: &Post) -> ();    
    async fn update(&self, user_id: &Uuid, followers: &Vec<Uuid>, post: &Post) -> ();    
    async fn delete(&self, user_id: &Uuid, followers: &Vec<Uuid>, post_id: &Uuid) -> ();    
}

pub struct FollowersServiceImpl<F, S> 
where 
    F: FriendRepository,    
    S: PostService {            
    repository: F,
    service: S,
    listeners: Vec<Arc<dyn PostListener + Send + Sync>>
}

impl <F, S> FollowersServiceImpl<F, S>
where 
    F: FriendRepository + Send + Sync,
    S: PostService + Send + Sync {
    pub fn new(repository: F, service: S, listeners: Vec<Arc<dyn PostListener + Send + Sync>>) -> Self {
        FollowersServiceImpl { 
            repository,
            service,
            listeners
        }
    }

    async fn fetch_followers(&self, user_id: Uuid) -> Result<Vec<Uuid>, FollowersServiceError> {
        Ok(self.repository.get_followers_ids(user_id).await?)
    }
}

#[async_trait]
impl <F, S> PostService for FollowersServiceImpl<F, S>
where 
    F: FriendRepository + Send + Sync,    
    S: PostService + Send + Sync {
    async fn create(&self, user_id: Uuid, text: &String) -> Result<Post, PostServiceError> {        
        let post = self.service.create(user_id, text).await?;   
        match self.fetch_followers(user_id).await {
            Ok(followers) => {
                for listener in &self.listeners {
                    listener.create(&user_id, &followers, &post).await;
                }
            },
            Err(e) => warn!("Failed to fetch followers {:?}", e)
        };        
        Ok(post)
    }

    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<Post, PostServiceError> {                        
        let post = self.service.update(user_id, post_id, text).await?;   
        match self.fetch_followers(user_id).await {
            Ok(followers) => {
                for listener in &self.listeners {
                    listener.update(&user_id, &followers, &post).await;
                }
            },
            Err(e) => warn!("Failed to fetch followers {:?}", e)
        };        
        Ok(post)
    }

    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError> {
        let _ = self.service.delete(user_id, post_id).await?;
        match self.fetch_followers(user_id).await {
            Ok(followers) => {
                for listener in &self.listeners {
                    listener.delete(&user_id, &followers, &post_id).await;
                }
            },
            Err(e) => warn!("Failed to fetch followers {:?}", e)
        };        
        Ok(())
    }

    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError> {
        Ok(self.service.get(post_id).await?)
    }

    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
        Ok(self.service.feed(user_id, limit, offset).await?)
    }
}
