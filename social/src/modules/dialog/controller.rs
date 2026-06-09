use openapi::apis::dialog::{Dialog, DialogUserIdListGetResponse, DialogUserIdSendPostResponse};
use axum_extra::headers::Host;
use axum_extra::extract::CookieJar;
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self};
use crate::modules::auth::auth;
use crate::Application;
use uuid::Uuid;

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
        Err(())
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
        Err(())
    }
}