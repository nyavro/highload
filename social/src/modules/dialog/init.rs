use async_trait::async_trait; 
use thiserror::Error;
use crate::modules::dialog::{repository::DialogRepositoryImpl, service::DialogServiceImpl, domain_models};
use uuid::Uuid;
use std::sync::Arc;
use deadpool_postgres::{Object};

#[async_trait]
pub trait DialogService {
    async fn send_message(&self, from: Uuid, to: Uuid, text: &String) -> Result<Uuid, DialogServiceError>;
    async fn list_messages(&self, from: Uuid, to: Uuid) -> Result<Vec<domain_models::DialogMessage>, DialogServiceError>;
}

pub fn new (client: Arc<Object>) -> DialogServiceImpl<DialogRepositoryImpl> {
    DialogServiceImpl::new(
        DialogRepositoryImpl::new(client)
    )
}

#[derive(Error, Debug)]
pub enum DialogServiceError {
    #[error("Database error: {0}")]
    Database(#[from] DialogRepositoryError)
}

#[derive(Error, Debug)]
pub enum DialogRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),
    
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait DialogRepository {
    async fn send(&self, from: Uuid, to: Uuid, text: &String) -> Result<Uuid, DialogRepositoryError>;   
    async fn list(&self, from: Uuid, to: Uuid) -> Result<Vec<domain_models::DialogMessage>, DialogRepositoryError>;   
}