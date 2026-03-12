use crate::modules::{common::ws::ws_manager::WebSocketManager, friend::repository::FriendRepositoryImpl, post::{async_notifier::AsyncNotifier, cached_post_service::CachedPostService, caching_listener::CachingPostListener, followers_service::{FollowersServiceError, FollowersServiceImpl, PostListener}, model::Post, post_cache::PostCacheImpl, post_service::PostServiceImpl, repository::{PostRepositoryError, PostRepositoryImpl}}};
use std::sync::Arc;
use fred::prelude;
use deadpool_postgres;
use thiserror::Error;
use uuid::Uuid;
use log::{error};
use async_trait::async_trait; 

#[derive(Error, Debug)]
pub enum PostServiceError {
    #[error("Database error: {0}")]
    Database(#[from] PostRepositoryError),
    #[error("Followers error: {0}")]
    Followers(#[from] FollowersServiceError),
}

#[async_trait]
pub trait PostService {
    async fn create(&self, user_id: Uuid, text: &String) -> Result<Post, PostServiceError>;
    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<Post, PostServiceError>;
    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError>;
    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError>;
    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError>;
}

pub fn create_service(pool: Arc<deadpool_postgres::Pool>, redis: Arc<prelude::Pool>, ws_manager: Arc<WebSocketManager>) -> Arc<dyn PostService + Send + Sync> {    
    let service = PostServiceImpl::new(
        PostRepositoryImpl::new(Arc::clone(&pool))
    );    
    let cached_service = CachedPostService::new(
        service,
        PostCacheImpl::new(Arc::clone(&redis))
    );
    let listeners: Vec<Arc<dyn PostListener + Send + Sync>> = vec!(
        Arc::new(CachingPostListener::new(PostCacheImpl::new(Arc::clone(&redis)))),
        Arc::new(AsyncNotifier::new(ws_manager))
    );
    let followers_service = FollowersServiceImpl::new(
        FriendRepositoryImpl::new(pool),        
        cached_service,
        listeners
    );
    Arc::new(followers_service)
}