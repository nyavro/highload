use uuid::Uuid;
use crate::modules::friend::repository::{FriendRepository, FriendRepositoryError};
use crate::modules::post::model::Post;
use crate::modules::post::service_provider::{PostService, PostServiceError};
use thiserror::Error;
use log::{error, warn, info};
use async_trait::async_trait;
use std::sync::Arc; 

#[derive(Error, Debug)]
pub enum FollowersServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] FriendRepositoryError)
}

#[derive(Clone, Debug)]
enum DomainEvent {
    PostCreated {
        user_id: Uuid,
        post: Post,
        followers: Vec<Uuid>,
    },
    PostUpdated {
        user_id: Uuid,
        post: Post,
        followers: Vec<Uuid>,
    },
    PostDeleted {
        user_id: Uuid,
        post_id: Uuid,
        followers: Vec<Uuid>,
    },
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
    event_bus: EventBus
}

impl <F, S> FollowersServiceImpl<F, S>
where 
    F: FriendRepository + Send + Sync,
    S: PostService + Send + Sync {
    pub fn new(repository: F, service: S, listeners: Vec<Arc<dyn PostListener + Send + Sync>>) -> Self {
        FollowersServiceImpl { 
            repository,
            service,
            event_bus: EventBus::new(100, listeners)
        }
    }

    async fn fetch_followers(&self, user_id: Uuid) -> Result<Vec<Uuid>, FollowersServiceError> {
        Ok(self.repository.get_followers_ids(user_id).await?)
    }
}

struct EventBus {
    sender: tokio::sync::mpsc::Sender<DomainEvent>,
}

impl EventBus {
    pub fn new(capacity: usize, listeners: Vec<Arc<dyn PostListener + Send + Sync>>) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel(capacity);
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {                
                let futures = listeners.iter().map(|l| {
                    match &event {
                        DomainEvent::PostCreated { user_id, post, followers } => l.create(&user_id, &followers, &post),                
                        DomainEvent::PostUpdated { user_id, post, followers } => l.update(&user_id, &followers, &post),
                        DomainEvent::PostDeleted { user_id, post_id, followers } => l.delete(&user_id, &followers, &post_id)
                    }
                });
                futures::future::join_all(futures).await;                
            }
        });

        Self { sender: tx }
    }

    async fn publish(&self, event: DomainEvent) {
        if let Err(e) = self.sender.send(event).await {
            info!("Failed to publish event: {:?}", e);
        }
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
                self.event_bus.publish(DomainEvent::PostCreated {
                    user_id,
                    post: post.clone(),
                    followers,
                }).await;
            },
            Err(e) => warn!("Failed to fetch followers {:?}", e)
        };                
        Ok(post)
    }

    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<Post, PostServiceError> {                        
        let post = self.service.update(user_id, post_id, text).await?;   
        match self.fetch_followers(user_id).await {
            Ok(followers) => {
                self.event_bus.publish(DomainEvent::PostUpdated {
                    user_id,
                    post: post.clone(),
                    followers,
                }).await;
            },
            Err(e) => warn!("Failed to fetch followers {:?}", e)
        };        
        Ok(post)
    }

    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError> {
        let _ = self.service.delete(user_id, post_id).await?;
        match self.fetch_followers(user_id).await {
            Ok(followers) => {
                self.event_bus.publish(DomainEvent::PostDeleted {
                    user_id,
                    post_id,
                    followers,
                }).await;
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
