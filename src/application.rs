use openapi::apis::default::{Default, UserRegisterPostResponse, LoginPostResponse, UserGetIdGetResponse};
use openapi::apis::ErrorHandler;
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self, User};
use std::sync::Arc;
use crate::app_state::AppState;
use crate::user_service;

#[derive(Clone)]
pub struct Application {
    state: Arc<AppState>, 
}

impl AsRef<Application> for Application {
    fn as_ref(&self) -> &Application {
        &self
    }
}

impl Application {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl ErrorHandler for Application {}

#[async_trait]
impl Default for Application {

    async fn login_post(
        &self,    
        _: &Method,
        _: &Host,
        _: &CookieJar,
        _: &Option<models::LoginPostRequest>
    ) -> Result<LoginPostResponse, ()> {
        Ok(
            LoginPostResponse::Status200(
                models::LoginPost200Response{token: Some("use this Token, Luke!".to_string())}
            )
        )
    }

    async fn user_get_id_get(
        &self,
        _: &Method,
        _: &Host,
        _: &CookieJar,
        path_params: &models::UserGetIdGetPathParams
    ) -> Result<UserGetIdGetResponse, ()> {
        let id = path_params.id.clone();
        let client = self.state.pool.get().await.unwrap();
        let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
        let rows = client.query(&stmt, &[&2]).await.unwrap();
        let value: i32 = rows[0].get(0);            
        Ok(UserGetIdGetResponse::Status200(
            User{id: Some(id),
                 first_name: Some("first_name_test".to_string()),
                 second_name: Some("second_name_test".to_string()),
                 birthdate: None,
                 biography: Some("Basics)".to_string() + &value.to_string()),
                 city: Some("city".to_string())}
            )
        )
    }

    async fn user_register_post(
        &self,    
        _: &Method,
        _: &Host,
        _: &CookieJar,
        user_registration_request: &Option<models::UserRegisterPostRequest>
    ) -> Result<UserRegisterPostResponse, ()> {                
        Ok(
            match user_registration_request {                
                Some(req) => {
                    let client = self.state.pool.get().await.unwrap();
                    let res = user_service::register_user(
                        client,
                        user_service::UserRegistration {
                            first_name: &req.first_name,
                            second_name: &req.second_name,
                            birthdate: &req.birthdate,
                            biography: &req.biography,
                            city: &req.city,
                            password: &req.password,
                        }
                    ).await;
                    UserRegisterPostResponse::Status200(
                        models::UserRegisterPost200Response {
                            user_id: res.user_id
                        }
                    )
                },
                None => UserRegisterPostResponse::Status400
            }            
        )
    }
}