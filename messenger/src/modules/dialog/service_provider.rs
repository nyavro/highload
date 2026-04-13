use std::sync::Arc;
use crate::modules::{common::tarantool::tarantool_manager::TarantoolManager, dialog::{service::{DialogServiceError, DialogServiceImpl}, tarantool::repository::DialogRepositoryImpl}}; 
use async_trait::async_trait; 
use crate::modules::dialog::domain_models;
use uuid::Uuid;
use deadpool::managed::Pool;
use thiserror::Error;

#[async_trait]
pub trait DialogService {
    async fn send_message(&self, from: Uuid, to: Uuid, text: &String) -> Result<Uuid, DialogServiceError>;
    async fn list_messages(&self, from: Uuid, to: Uuid) -> Result<Vec<domain_models::DialogMessage>, DialogServiceError>;
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
    async fn list(&self, from: Uuid, to: Uuid, offset: u32, limit: u32) -> Result<Vec<domain_models::DialogMessage>, DialogRepositoryError>;   
}

pub fn create_service(tarantool_pool: Arc<Pool<TarantoolManager>>) 
    -> Arc<dyn DialogService + Send + Sync> {  
    let repository =  DialogRepositoryImpl::new(tarantool_pool);
    let service = DialogServiceImpl::new(repository);
    Arc::new(
        service
    )
}