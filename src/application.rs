use chrono::NaiveDate;
use openapi::apis::default::{Default, UserRegisterPostResponse, LoginPostResponse, UserGetIdGetResponse};
use openapi::apis::ErrorHandler;
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self, User};
use std::sync::Arc;
use crate::app_state::AppState;
use crate::user_service;
use uuid::Uuid;

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
        login: &Option<models::LoginPostRequest>
    ) -> Result<LoginPostResponse, ()> {
        let login = login.clone().expect("No login passed");
        let client = self.state.pool.get().await.unwrap();
        match user_service::authenticate_user(
            client,
            Uuid::parse_str(&login.id.expect("Login must contain user id")).unwrap(),
            login.password.expect("Login must contain user password"),
        ).await {
            Ok(res) => Ok(
                if res { 
                    LoginPostResponse::Status200(models::LoginPost200Response{token: Some("use this Token, Luke!".to_string())}) }
                else {
                    LoginPostResponse::Status400    
                }
            ),
            Err(_) => Ok(
                LoginPostResponse::Status400
            )
        }        
    }

    async fn user_get_id_get(
        &self,
        _: &Method,
        _: &Host,
        _: &CookieJar,
        path_params: &models::UserGetIdGetPathParams
    ) -> Result<UserGetIdGetResponse, ()> {
        let id = Uuid::parse_str(&path_params.id).unwrap();
        let client = self.state.pool.get().await.unwrap();
        let stmt = client.prepare_cached("SELECT first_name, second_name, birthdate, biography, city FROM users WHERE id=$1").await.unwrap();
        let row = client.query_one(&stmt, &[&id]).await.unwrap();
        let first_name: String = row.get(0);            
        let second_name: String = row.get(1);            
        let birthdate: Option<NaiveDate> = row.get(2);            
        let biography: Option<String> = row.get(3);            
        let city: Option<String> = row.get(4);            
        Ok(UserGetIdGetResponse::Status200(
            User{id: Some(id.to_string()),
                 first_name: Some(first_name),
                 second_name: Some(second_name),
                 birthdate,
                 biography,
                 city}
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
                    match res {
                        Ok(r) => UserRegisterPostResponse::Status200(
                            models::UserRegisterPost200Response {
                                user_id: r.user_id
                                }
                            ),
                        Err(_) => UserRegisterPostResponse::Status400
                    }
                },
                None => UserRegisterPostResponse::Status400
            }            
        )
    }
}