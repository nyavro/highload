use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::modules::post::model::Post;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DomainEvent {
    PostCreated {
        user_id: Uuid,
        post: Post,        
    },
    PostUpdated {
        user_id: Uuid,
        post: Post,        
    },
    PostDeleted {
        user_id: Uuid,
        post_id: Uuid,        
    },
}

impl DomainEvent {
    pub fn user_id(&self) -> &Uuid {
        match self {
            DomainEvent::PostCreated {user_id, ..} => user_id,
            DomainEvent::PostUpdated {user_id, ..} => user_id,
            DomainEvent::PostDeleted {user_id, ..} => user_id
        }        
    }
}
