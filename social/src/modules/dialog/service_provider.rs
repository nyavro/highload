use std::sync::Arc;
use thiserror::Error;
use log::{error};
use async_trait::async_trait;
use messenger_client::apis::configuration::Configuration;

use crate::modules::dialog::{models, service::DialogServiceImpl}; 

#[derive(Error, Debug)]
pub enum DialogServiceError {
    #[error("Integration error: {0}")]
    Integration(String),    
}

#[async_trait]
pub trait DialogService {  
    async fn list_messages(&self, user_id: &String) -> Result<Vec<models::Message>,DialogServiceError>;  
    async fn send(&self, to_user_id: &String, text: String) -> Result<(), DialogServiceError>;
}

pub fn create_service(config: Arc<Configuration>) -> Arc<dyn DialogService + Send + Sync> {        
    Arc::new(DialogServiceImpl::new(config))
}