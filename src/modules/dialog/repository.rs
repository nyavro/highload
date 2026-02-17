use uuid::Uuid;
use deadpool_postgres::{Object};
use std::sync::Arc;
use async_trait::async_trait; 
use crate::modules::dialog::{init::{DialogRepositoryError, DialogRepository}, domain_models};

pub struct DialogRepositoryImpl {
    client: Arc<Object>
}

impl DialogRepositoryImpl {
    pub fn new(client: Arc<Object>) -> Self {
        DialogRepositoryImpl { client }
    }
}

#[async_trait]
impl DialogRepository for DialogRepositoryImpl {    

    async fn send(&self, from: Uuid, to: Uuid, text: &String) -> Result<Uuid, DialogRepositoryError> {
        let res = self.client.query_one(
            "INSERT INTO dialogs (from_id, to_id, text) VALUES ($1, $2, $3) RETURNING id", 
            &[&from, &to, text]
        ).await?;
        let id: Uuid = res.get("id");        
        Ok(id)
    }

    async fn list(&self, from: Uuid, to: Uuid) -> Result<Vec<domain_models::DialogMessage>, DialogRepositoryError> {        
        let res = self.client.query(
            "SELECT from_id, to_id, text FROM dialogs WHERE from_id=$1 AND to_id=$2 OR from_id=$2 AND to_id=$1 ORDER BY created_at DESC", 
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