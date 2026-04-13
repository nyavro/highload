use async_trait::async_trait; 
use uuid::Uuid;
use log::info;
use crate::modules::dialog::{domain_models, service_provider::{DialogRepository, DialogRepositoryError, DialogService}};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DialogServiceError {
    #[error("Database error: {0}")]
    Database(#[from] DialogRepositoryError)
}

pub struct DialogServiceImpl<R> 
where
    R: DialogRepository {
    repository: R
} 

impl <R> DialogServiceImpl<R> 
where R: DialogRepository {
    pub fn new(repository: R) -> Self {
        DialogServiceImpl {repository}
    }
}

#[async_trait]
impl <R> DialogService for DialogServiceImpl<R> 
where R: DialogRepository + Send + Sync {

    async fn send_message(&self, from: Uuid, to: Uuid, text: &String) -> Result<Uuid, DialogServiceError> {
        info!("Dialog service processing message post");
        Ok(self.repository.send(from, to, text).await?)
    }

    async fn list_messages(&self, from: Uuid, to: Uuid) -> Result<Vec<domain_models::DialogMessage>, DialogServiceError> {
        Ok(self.repository.list(from, to, 0, 100).await?)
    }
}