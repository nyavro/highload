use std::sync::Arc;
use fred::prelude;
use deadpool_postgres;
use thiserror::Error;
use uuid::Uuid;
use log::{error};
use async_trait::async_trait;

use crate::modules::post::{cached_post_service::CachedPostService, model::Post, post_cache::PostCacheImpl, post_service::PostServiceImpl, publishing_service::PublishingServiceImpl, rabbitmq::RabbitPublisher, repository::{PostRepositoryError, PostRepositoryImpl}}; 

#[derive(Error, Debug)]
pub enum PostServiceError {
    #[error("Database error: {0}")]
    Database(#[from] PostRepositoryError),

    #[error("Error: {0}")]
    Inner(#[from] Box<dyn std::error::Error>),
}

#[async_trait]
pub trait PostService {
    async fn create(&self, user_id: Uuid, text: &String) -> Result<Post, PostServiceError>;
    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<Post, PostServiceError>;
    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError>;
    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError>;
    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError>;
}

pub fn create_service(pool: Arc<deadpool_postgres::Pool>, redis: Arc<prelude::Pool>, rabbitmq: Arc<deadpool_lapin::Pool>, exchange: String) -> Arc<dyn PostService + Send + Sync> {    
    let service = PostServiceImpl::new(
        PostRepositoryImpl::new(Arc::clone(&pool))
    );    
    let cached_service = CachedPostService::new(
        service,
        PostCacheImpl::new(Arc::clone(&redis))
    );
    let publishing_service = PublishingServiceImpl::new(
        cached_service, 
        Arc::new(RabbitPublisher::new(rabbitmq, exchange)), 
    );
    Arc::new(publishing_service)
}