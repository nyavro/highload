use openapi::apis::auth::{Auth, LoginPostResponse};
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self};
use crate::modules::auth::auth;
use crate::modules::auth::auth_service;
use uuid::Uuid;
use crate::Application;

#[async_trait]
impl Auth for Application {

    async fn login_post(
        &self,    
        _: &Method,
        _: &Host,
        _: &CookieJar,
        login: &Option<models::LoginPostRequest>
    ) -> Result<LoginPostResponse, ()> {
        let login_data = login.as_ref().ok_or(())?;        
        let uuid = match Uuid::parse_str(&login_data.id) {
            Ok(id) => id,
            Err(_) => return Ok(LoginPostResponse::Status400)
        };
        match auth_service::authenticate_user(
            self.state.get_master_client().await,
            &uuid,
            &login_data.password,
        ).await {
            Ok(true) => 
                match auth::create_token(&uuid, self.state.secret.as_bytes(), self.state.jwt_token_ttl_minutes) {
                    Ok(token) => Ok(LoginPostResponse::Status200(models::LoginPost200Response{token: Some(token)})),
                    Err(_) => Ok(LoginPostResponse::Status400)
                },
            _ => Ok(
                LoginPostResponse::Status400
            )
        }        
    }
}