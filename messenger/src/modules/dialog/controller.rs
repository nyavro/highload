use openapi::apis::{dialog::{Dialog, DialogUserIdListGetResponse, DialogUserIdSendPostResponse}};
use openapi::models::{self, DialogMessage};
use crate::Application;
use async_trait::async_trait; 
use axum_extra::headers::Host;
use axum_extra::extract::CookieJar;
use axum::http::Method;
use crate::modules::auth::auth;
use crate::modules::dialog;
use uuid::Uuid;
use log::info;

#[async_trait]
impl Dialog for Application {
    type Claims = auth::Claims;

    async fn dialog_user_id_list_get(
        &self,
        _: &Method,
        _: &Host,
        _: &CookieJar,
        claims: &Self::Claims,
        path_params: &models::DialogUserIdListGetPathParams,
    ) -> Result<DialogUserIdListGetResponse, ()> {
        let user_id = match Uuid::parse_str(&path_params.user_id) {
            Ok(id) => id,
            Err(e) => {
                info!("Could not parse: {:?}", e);
                return Ok(DialogUserIdListGetResponse::Status400);
            }
        };  
        match self.state.dialog_service.list_messages(claims.user_id, user_id).await {
            Ok(res) => Ok(DialogUserIdListGetResponse::Status200(to_dtos(res))),
            Err(e) => {
                info!("Failure: {:?}", e);
                Err(())
            }
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
        let user_id = match Uuid::parse_str(&path_params.user_id) {
            Ok(id) => id,
            Err(_) => return Ok(DialogUserIdSendPostResponse::Status400)
        };    
        if let Some(message) = body {
            match self.state.dialog_service.send_message(claims.user_id, user_id, &message.text).await {
                Ok(_) => Ok(DialogUserIdSendPostResponse::Status200),
                Err(err) => Err(())
            }
        } else {
            Ok(DialogUserIdSendPostResponse::Status400)
        }
    }
}

fn to_dtos(vec: Vec<dialog::domain_models::DialogMessage>) -> Vec<DialogMessage> {
    vec.into_iter().map(to_dto).collect()
}

fn to_dto(item: dialog::domain_models::DialogMessage) -> DialogMessage {
    DialogMessage {
        from: item.from.to_string(),    
        to: item.to.to_string(),
        text: item.text
    }
}