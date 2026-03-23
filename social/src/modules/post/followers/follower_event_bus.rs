use serde::Serialize;
use uuid::Uuid;
use crate::modules::post::event::DomainEvent;
use crate::modules::post::model::Post;
use log::{info};
use async_trait::async_trait;
use std::sync::Arc; 

#[derive(Clone, Debug, Serialize)]
pub struct FollowerEvent {
    pub domain_event: DomainEvent,   
    pub followers: Vec<Uuid>
}

#[async_trait]
pub trait FollowerEventListener {
    async fn create(&self, user_id: &Uuid, followers: &Vec<Uuid>, post: &Post) -> ();    
    async fn update(&self, user_id: &Uuid, followers: &Vec<Uuid>, post: &Post) -> ();    
    async fn delete(&self, user_id: &Uuid, followers: &Vec<Uuid>, post_id: &Uuid) -> ();    
}

pub struct EventBus {
    sender: tokio::sync::mpsc::Sender<FollowerEvent>,
}

impl EventBus {
    pub fn new(capacity: usize, listeners: Vec<Arc<dyn FollowerEventListener + Send + Sync>>) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<FollowerEvent>(capacity);
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {                
                let futures = listeners.iter().map(|l| {
                    match &event.domain_event {
                        DomainEvent::PostCreated { user_id, post} => l.create(&user_id, &event.followers, &post),                
                        DomainEvent::PostUpdated { user_id, post} => l.update(&user_id, &event.followers, &post),
                        DomainEvent::PostDeleted { user_id, post_id} => l.delete(&user_id, &event.followers, &post_id)
                    }
                });
                futures::future::join_all(futures).await;                
            }
        });

        Self { sender: tx }
    }

    pub async fn publish(&self, event: FollowerEvent) {
        if let Err(e) = self.sender.send(event).await {
            info!("Failed to publish event: {:?}", e);
        }
    }
}
