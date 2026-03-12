use crate::modules::{common::ws::ws_manager::WebSocketManager, post::{followers_service::PostListener, model::Post}};
use uuid::Uuid;
use async_trait::async_trait; 
use std::sync::Arc;
use serde::Serialize;

pub struct AsyncNotifier {    
    ws_manager: Arc<WebSocketManager>
}

#[derive(Debug, Clone, Serialize)]
enum PostEvent {
    Create,
    Update,
    Delete
}

#[derive(Debug, Clone, Serialize)]
struct PostNotification {
    pub event: PostEvent,
    pub post_id: Uuid,
    pub text: Option<String>
}

impl AsyncNotifier {
    pub fn new(ws_manager: Arc<WebSocketManager>) -> Self {
        AsyncNotifier { 
            ws_manager 
        }
    }
}

#[async_trait]
impl PostListener for AsyncNotifier {    
    async fn create(&self, _: &Uuid, followers: &Vec<Uuid>, post: &Post) {
        let _ = self.ws_manager.send_to_users(
            followers, 
            &PostNotification {
                event: PostEvent::Create, 
                post_id: 
                post.id.clone(), 
                text: Some(post.text.clone())
            }
        );
    }    
    async fn update(&self, _: &Uuid, followers: &Vec<Uuid>, post: &Post) {
        let _ = self.ws_manager.send_to_users(
            followers, 
            &PostNotification {
                event: PostEvent::Update, 
                post_id: 
                post.id.clone(), 
                text: Some(post.text.clone())
            }
        );
    }   
    async fn delete(&self, _: &Uuid, followers: &Vec<Uuid>, post_id: &Uuid) {
        let _ = self.ws_manager.send_to_users(
            followers, 
            &PostNotification {
                event: PostEvent::Delete, 
                post_id: *post_id,
                text: None
            }
        );
    }
}