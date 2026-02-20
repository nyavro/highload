use uuid::Uuid;
use deadpool_postgres::Pool;
use std::sync::Arc;
use async_trait::async_trait; 
use crate::modules::dialog::{init::{DialogRepositoryError, DialogRepository}, domain_models};
use log::{info, error};

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
        info!("Saving message to DB");
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;    
        let message_id = Uuid::now_v7();
        info!("Saving sender part: {:?}", message_id);
        match tx.execute(
            "INSERT INTO dialogs (owner_id, from_id, to_id, message_id, text) VALUES ($1, $2, $3, $4, $5)", 
            &[&from, &from, &to, &message_id, text]
        ).await {
            Ok(res) => info!("Succeeded"),
            Err(e) => error!("Failed to save message {:?}", e)
        };
        info!("Sender part done");    
        if from != to {
            info!("Saving receiver part");
            tx.execute(
                "INSERT INTO dialogs (owner_id, from_id, to_id, message_id, text) VALUES ($1, $2, $3, $4, $5)", 
                &[&to, &from, &to, &message_id, text]
            ).await?;
            info!("Saving receiver done");
        }
        info!("Closing transaction");
        tx.commit().await?;            
        info!("Closing transaction done");
        Ok(message_id)
    }

    async fn list(&self, from: Uuid, to: Uuid) -> Result<Vec<domain_models::DialogMessage>, DialogRepositoryError> {        
        let client = self.pool.get().await?;
        let res = client.query(
            "SELECT from_id, to_id, text FROM dialogs WHERE owner_id=$1 AND to_id=$2 ORDER BY created_at DESC", 
            &[&from, &to]
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