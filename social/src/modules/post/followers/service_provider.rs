use std::sync::Arc;
use fred::prelude;
use deadpool_postgres;
use crate::modules::{common::ws::ws_manager::WebSocketManager, friend::repository::FriendRepositoryImpl, post::{followers::{async_notifier::AsyncNotifier, caching_listener::CachingPostListener, follower_event_bus::FollowerEventListener, followers_service::{FollowersService, FollowersServiceImpl}}, post_cache::PostCacheImpl}}; 


pub fn create_service(pool: Arc<deadpool_postgres::Pool>, redis: Arc<prelude::Pool>, rabbitmq: Arc<deadpool_lapin::Pool>, ws_manager: Arc<WebSocketManager>, exchange: String) 
    -> Arc<dyn FollowersService + Send + Sync> {        
    let listeners: Vec<Arc<dyn FollowerEventListener + Send + Sync>> = vec!(        
        Arc::new(CachingPostListener::new(PostCacheImpl::new(Arc::clone(&redis)))),
        Arc::new(AsyncNotifier::new(ws_manager))
    );
    let followers_service = FollowersServiceImpl::new(
        FriendRepositoryImpl::new(pool),                
        listeners,
        rabbitmq,
        exchange
    );    
    Arc::new(followers_service)
}