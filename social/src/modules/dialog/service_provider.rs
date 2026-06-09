use std::sync::Arc;
use thiserror::Error;
use log::{error};
use async_trait::async_trait;

use crate::modules::{dialog::service::DialogServiceImpl}; 

#[derive(Error, Debug)]
pub enum DialogServiceError {
    #[error("Error: {0}")]
    Inner(#[from] Box<dyn std::error::Error>),
}

#[async_trait]
pub trait DialogService {
}

pub fn create_service() -> Arc<dyn DialogService + Send + Sync> {        
    Arc::new(DialogServiceImpl::new())
}