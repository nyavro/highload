use async_trait::async_trait; 
use crate::modules::dialog::{init::{DialogService, DialogRepository, DialogServiceError}, domain_models};
use uuid::Uuid;

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
        Ok(self.repository.send(from, to, text).await?)
    }

    async fn list_messages(&self, from: Uuid, to: Uuid) -> Result<Vec<domain_models::DialogMessage>, DialogServiceError> {
        Ok(self.repository.list(from, to).await?)
    }
}