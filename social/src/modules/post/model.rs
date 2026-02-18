use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub id: Uuid,            
    pub text: String,
    pub author_user_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>
}
