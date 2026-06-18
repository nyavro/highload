use openapi::apis::dialog::{Dialog, DialogUserIdListGetResponse, DialogUserIdSendPostResponse};
use axum_extra::headers::Host;
use axum_extra::extract::CookieJar;
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self, DialogMessage};
use crate::modules::auth::auth;
use crate::Application;
use crate::modules::dialog::models::Message;

#[async_trait]
impl Dialog for Application {
    type Claims = auth::Claims;

    async fn dialog_user_id_list_get(
        &self,    
        _: &Method,
        host: &Host,
        _: &CookieJar,
        claims: &Self::Claims,
        path_params: &models::DialogUserIdListGetPathParams,
    ) -> Result<DialogUserIdListGetResponse, ()> {
        match self.state.dialog_service.list_messages(&path_params.user_id).await {
            Ok(res) => Ok(DialogUserIdListGetResponse::Status200(to_dto_messages(res))),
            Err(e) => Err(())
        }            
    }

    async fn dialog_user_id_send_post(
        &self,    
        _: &Method,
        _: &Host,
        _: &CookieJar,
        claims: &Self::Claims,
        path_params: &models::DialogUserIdSendPostPathParams,
        body: &Option<models::DialogUserIdSendPostRequest>,
    ) -> Result<DialogUserIdSendPostResponse, ()> {        
        if let Some(send) = body {            
            match self.state.dialog_service.send(&path_params.user_id, send.text.clone()).await {
                Ok(_) => Ok(DialogUserIdSendPostResponse::Status200),
                Err(_) => Err(())
            }
        } else {
            Err(())
        }        
    }
}

fn to_dto_message(message: Message) -> DialogMessage {
    DialogMessage {
        from: message.from,
        to: message.to,
        text: message.text
    }
}

fn to_dto_messages(messages: Vec<Message>) -> Vec<DialogMessage> {
    messages.into_iter().map(|m| to_dto_message(m)).collect()
}