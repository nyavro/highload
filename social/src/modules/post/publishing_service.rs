use log::info;
use uuid::Uuid;
use crate::modules::post::event::DomainEvent;
use crate::modules::post::model::Post;
use crate::modules::post::rabbitmq::RabbitPublisher;
use crate::modules::post::service_provider::{PostService, PostServiceError};
use async_trait::async_trait;
use std::sync::Arc; 

pub struct PublishingServiceImpl <S> 
where 
    S: PostService {
    rabbit_publisher: Arc<RabbitPublisher>,
    service: S,
}

impl <S> PublishingServiceImpl<S>
where 
    S: PostService + Send + Sync {
    pub fn new(service: S, rabbit_publisher: Arc<RabbitPublisher>) -> Self {
        PublishingServiceImpl { 
            rabbit_publisher,
            service
        }
    }
}

#[async_trait]
impl <S> PostService for PublishingServiceImpl<S>
where 
    S: PostService + Send + Sync {
    async fn create(&self, user_id: Uuid, text: &String) -> Result<Post, PostServiceError> {        
        let post = self.service.create(user_id, text).await?; 
        info!("Create post at PublishingService");
        let _ = self.rabbit_publisher.publish(
            &DomainEvent::PostCreated {
                user_id,
                post: post.clone(),                
            }
        ).await?;          
        Ok(post)
    }

    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<Post, PostServiceError> {                        
        let post = self.service.update(user_id, post_id, text).await?;   
        let _ = self.rabbit_publisher.publish(
            &DomainEvent::PostUpdated {
                user_id,
                post: post.clone(),                
            }
        ).await?;
        Ok(post)
    }

    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError> {
        let _ = self.service.delete(user_id, post_id).await?;
        let _ = self.rabbit_publisher.publish(
            &DomainEvent::PostDeleted {
                user_id,
                post_id
            }
        ).await?;     
        Ok(())
    }

    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError> {
        Ok(self.service.get(post_id).await?)
    }

    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
        Ok(self.service.feed(user_id, limit, offset).await?)
    }
}
