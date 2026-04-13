use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait; 
use crate::modules::{common::tarantool::{tarantool_manager::TarantoolManager}, dialog::{domain_models, service_provider::{DialogRepository, DialogRepositoryError}}};
use log::{info};
use deadpool::managed::{Pool};
use tarantool_rs::ExecutorExt;

pub struct DialogRepositoryImpl {
    pool: Arc<Pool<TarantoolManager>>
}

impl DialogRepositoryImpl {
    pub fn new(pool: Arc<Pool<TarantoolManager>>) -> Self {
        DialogRepositoryImpl { pool }
    }
}

#[async_trait]
impl DialogRepository for DialogRepositoryImpl {    

    async fn send(&self, from: Uuid, to: Uuid, text: &String) -> Result<Uuid, DialogRepositoryError> {
        info!("Saving message to DB");
        let client = self.pool.get().await.map_err(|e| DialogRepositoryError::Internal(e.to_string()))?;
        info!("Got the client");
        let message_id = Uuid::now_v7();
        let result = client
            .call("send_message", (&from.to_string(), &from.to_string(), &to.to_string(), &message_id.to_string(), text))
            .await.map_err(|e| DialogRepositoryError::Internal(e.to_string()))?;
        let message_id_str = result
            .decode_first::<String>()
            .map_err(|e| DialogRepositoryError::Internal(e.to_string()))?;        
        info!("Got message id {:?}", message_id_str);
        // let message_id = string_to_uuid(&message_id_str).map_err(|e| DialogRepositoryError::Internal(e.to_string()))?;
        // info!("Message saved successfully with ID: {}", message_id);
        Ok(message_id)
    }

    async fn list(&self, from: Uuid, to: Uuid, offset: u32, limit: u32) -> Result<Vec<domain_models::DialogMessage>, DialogRepositoryError> {        
        let client = self.pool.get().await.map_err(|e| DialogRepositoryError::Internal(e.to_string()))?;
        let from_str = from.to_string();
        let to_str = to.to_string();        
        let messages_data: Vec<(String, String, String)> = client
            .call("list_dialogs", (&from_str, &to_str, &limit, &offset))
            .await
            .map_err(|e| DialogRepositoryError::Internal(e.to_string()))?
            .decode_first()
            .map_err(|e| DialogRepositoryError::Internal(format!("Failed to decode response: {}", e)))?;
        let res = messages_data
            .into_iter()
            .map(|(from_id, to_id, text)| {
                Ok(domain_models::DialogMessage {
                    from: Uuid::parse_str(&from_id).map_err(|e| DialogRepositoryError::Internal(e.to_string()))?,
                    to: Uuid::parse_str(&to_id).map_err(|e| DialogRepositoryError::Internal(e.to_string()))?,
                    text,
                })
            })
            .collect::<Result<Vec<_>, DialogRepositoryError>>();
        res
    }
}