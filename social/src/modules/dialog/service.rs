use std::sync::Arc;
use crate::modules::dialog::{models, service_provider::{DialogService, DialogServiceError}};
use messenger_client::{apis::{configuration::Configuration, dialog_api::{dialog_user_id_list_get, dialog_user_id_send_post}}, models::{DialogMessage, DialogUserIdSendPostRequest}};
use async_trait::async_trait;

pub struct DialogServiceImpl {
    config: Arc<Configuration>    
}

impl DialogServiceImpl {
    pub fn new(config: Arc<Configuration>) -> Self { 
        DialogServiceImpl {config}       
    }
}

#[async_trait]
impl DialogService for DialogServiceImpl {
    async fn list_messages(&self, user_id: &String) -> Result<Vec<models::Message>, DialogServiceError> {
        match dialog_user_id_list_get(&self.config, &user_id).await {
            Ok(res) => Ok(to_domain_messages(res)),
            Err(e) => Err(DialogServiceError::Integration(e.to_string()))
        }
    }
    async fn send(&self, to_user_id: &String, text: String) -> Result<(), DialogServiceError> {    
        match dialog_user_id_send_post(&self.config, &to_user_id, Some(DialogUserIdSendPostRequest {text})).await {
            Ok(response) => {
                println!("Send result: {:?}", response);
                Ok(())
            },
            Err(e) => {
                println!("Integration failure: {:?}", e);
                Err(DialogServiceError::Integration(e.to_string()))
            }
        }
    }
}

fn to_domain_message(message: DialogMessage) -> models::Message {
    models::Message {
        from: message.from,
        to: message.to,
        text: message.text
    }    
}

fn to_domain_messages(messages: Vec<DialogMessage>) -> Vec<models::Message> {
    messages.into_iter().map(|m| to_domain_message(m)).collect()
}