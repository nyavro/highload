use uuid::Uuid;
use deadpool_postgres::Pool;
use std::sync::Arc;
use async_trait::async_trait;
use crate::modules::dialog::{domain_models, service_provider::{DialogRepository, DialogRepositoryError}};

pub struct DialogRepositoryImpl {
    pool: Arc<Pool>
}

impl DialogRepositoryImpl {
    pub fn new(pool: Arc<Pool>) -> Self {
        DialogRepositoryImpl { pool }
    }
}

#[async_trait]
impl DialogRepository for DialogRepositoryImpl {    

    async fn send(&self, from: Uuid, to: Uuid, text: &String) -> Result<Uuid, DialogRepositoryError> {
        tracing::info!("Saving message to DB");
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;    
        let message_id = Uuid::now_v7();
        tracing::info!("Saving sender part: {:?}", message_id);
        match tx.execute(
            "INSERT INTO dialogs (owner_id, from_id, to_id, message_id, text) VALUES ($1, $2, $3, $4, $5)", 
            &[&from, &from, &to, &message_id, text]
        ).await {
            Ok(res) => tracing::info!("Succeeded"),
            Err(e) => tracing::error!("Failed to save message {:?}", e)
        };
        tracing::info!("Sender part done");    
        if from != to {
            tracing::info!("Saving receiver part");
            tx.execute(
                "INSERT INTO dialogs (owner_id, from_id, to_id, message_id, text) VALUES ($1, $2, $3, $4, $5)", 
                &[&to, &from, &to, &message_id, text]
            ).await?;
            tracing::info!("Saving receiver done");
        }
        tracing::info!("Closing transaction");
        tx.commit().await?;            
        tracing::info!("Closing transaction done");
        Ok(message_id)
    }

    async fn list(&self, from: Uuid, to: Uuid, offset: u32, limit: u32) -> Result<Vec<domain_models::DialogMessage>, DialogRepositoryError> {        
        let client = self.pool.get().await?;
        let res = client.query(
            "SELECT from_id, to_id, text FROM dialogs WHERE owner_id=$1 AND to_id=$2 ORDER BY created_at DESC OFFSET $3 LIMIT $4", 
            &[&from, &to, &offset, &limit]
        ).await?;        
        Ok(
            res.iter().map(|row| {
                let from: Uuid = row.get("from_id");    
                let to: Uuid = row.get("to_id");    
                let text: String = row.get("text");                
                domain_models::DialogMessage {from, to, text}
            }).collect()
        )    
    }
}