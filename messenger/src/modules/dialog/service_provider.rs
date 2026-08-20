use std::sync::Arc;
use crate::modules::dialog::{
    composite_repository::DialogRepositoryComposite, domain_models::DialogMessage, 
    postgres::repository::DialogRepositoryImpl as PostgresRepo, 
    tarantool::repository::DialogRepositoryImpl as TarantoolRepo,
    service::{DialogServiceError, DialogServiceImpl}, 
}; 
use async_trait::async_trait; 
use crate::modules::dialog::domain_models;
use uuid::Uuid;
use deadpool::managed::Pool as ManagedPool;
use deadpool_postgres::Pool;
use thiserror::Error;

#[async_trait]
pub trait DialogService {
    async fn send_message(&self, from: Uuid, to: Uuid, text: &String) -> Result<Uuid, DialogServiceError>;
    async fn list_messages(&self, from: Uuid, to: Uuid) -> Result<Vec<DialogMessage>, DialogServiceError>;
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

pub fn create_service(
    tarantool_pool: Arc<ManagedPool<crate::modules::common::tarantool::tarantool_manager::TarantoolManager>>,
    postgres_pool: Arc<Pool>,
) -> Arc<dyn DialogService + Send + Sync> {      
    let service = DialogServiceImpl::new(        
       DialogRepositoryComposite::new(TarantoolRepo::new(tarantool_pool), 
            PostgresRepo::new(postgres_pool)
        )
    );
    Arc::new(
        service
    )
}