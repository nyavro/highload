use openapi::apis::dialog::{Dialog, DialogUserIdListGetResponse, DialogUserIdSendPostResponse};
use openapi::models::{self};
use crate::Application;
use async_trait::async_trait; 
use axum_extra::headers::Host;
use axum_extra::extract::CookieJar;
use axum::http::Method;
use crate::modules::auth::auth;

#[async_trait]
impl Dialog for Application {
    type Claims = auth::Claims;

    async fn dialog_user_id_list_get(
        &self,
        _: &Method,
        _: &Host,
        _: &CookieJar,
        _: &Self::Claims,
        path_params: &models::DialogUserIdListGetPathParams,
    ) -> Result<DialogUserIdListGetResponse, ()> {
        Err(())
    }

    /// DialogUserIdSendPost - POST /dialog/{user_id}/send
    async fn dialog_user_id_send_post(
        &self,        
        _: &Method,
        _: &Host,
        _: &CookieJar,
        _: &Self::Claims,
        path_params: &models::DialogUserIdSendPostPathParams,
        body: &Option<models::DialogUserIdSendPostRequest>,
    ) -> Result<DialogUserIdSendPostResponse, ()> {
        Err(())
    }
}