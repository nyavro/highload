use async_trait::async_trait; 
use crate::modules::dialog::domain_models;
use uuid::Uuid;

use crate::modules::dialog::{service_provider::{DialogRepository, DialogRepositoryError}};

pub struct DialogRepositoryComposite <F,S> 
where F: DialogRepository,
      S: DialogRepository {
    fast: F,
    slow: S
}

impl <F,S> DialogRepositoryComposite<F, S> 
where F: DialogRepository,
      S: DialogRepository {
    pub fn new(fast: F, slow: S) -> Self {
        Self {fast, slow}
    }
}

#[async_trait]
impl <F,S> DialogRepository for DialogRepositoryComposite<F, S> 
where F: DialogRepository + Send + Sync ,
      S: DialogRepository + Send + Sync {
    async fn send(&self, from: Uuid, to: Uuid, text: &String) -> Result<Uuid, DialogRepositoryError> {
        let fast_res = self.fast.send(from, to, text).await;
        let slow_res = self.slow.send(from, to, text).await;
        match (fast_res, slow_res) {
            (Ok(id),_)| (_, Ok(id)) => Ok(id),
            (Err(tar_err), _) => {
                tracing::error!("Both repositories failed: {}", tar_err);
                Err(tar_err)
            }
        }
    } 

    async fn list(&self, from: Uuid, to: Uuid, offset: u32, limit: u32) -> Result<Vec<domain_models::DialogMessage>, DialogRepositoryError> {
        match self.fast.list(from, to, offset, limit).await {
            Ok(messages) => Ok(messages),
            Err(_) => {
                tracing::warn!("Tarantool unavailable, falling back to PostgreSQL");
                self.slow.list(from, to, offset, limit).await
            }
        }
    }
}