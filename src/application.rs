use chrono::NaiveDate;
use openapi::apis::default::{Default, UserRegisterPostResponse, LoginPostResponse, UserGetIdGetResponse};
use openapi::apis::{ApiAuthBasic, BasicAuthKind, ErrorHandler};
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self, User};
use std::sync::Arc;
use crate::app_state::AppState;
use crate::user_service;
use uuid::Uuid;
use crate::auth;
use axum::response::IntoResponse;
use axum::{http::{StatusCode}};
use log::{info};
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Clone)]
pub struct Application {
    state: Arc<AppState>, 
}

impl AsRef<Application> for Application {
    fn as_ref(&self) -> &Application {
        &self
    }
}

#[async_trait::async_trait]
impl ApiAuthBasic for Application {
    type Claims = auth::Claims;
    async fn extract_claims_from_auth_header(&self, kind: BasicAuthKind, headers: &axum::http::header::HeaderMap, key: &str) -> Option<Self::Claims> {
        // 1. Получаем заголовок Authorization
        let auth_header = headers.get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                (StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header").into_response()
            }).unwrap();

        // 2. Проверяем, что заголовок начинается с "Bearer "
        let token = auth_header.strip_prefix("Bearer ")
            .ok_or_else(|| {
                (StatusCode::UNAUTHORIZED, "Invalid authorization scheme, expected Bearer").into_response()
            }).unwrap();
        info!("{:?}", token);
        // 3. Считываем секретный ключ (лучше передавать через Axum State, но пока так)
        let secret = std::env::var("JWT_SECRET").unwrap();
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());

        // 4. Декодируем и валидируем токен
        let validation = Validation::default();
        let claims_data = decode::<auth::Claims>(token, &decoding_key, &validation)
            .map_err(|_| {
                (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response()
            }).unwrap();
        Some(claims_data.claims)
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
    type Claims = auth::Claims;

    async fn login_post(
        &self,    
        _: &Method,
        _: &Host,
        _: &CookieJar,
        login: &Option<models::LoginPostRequest>
    ) -> Result<LoginPostResponse, ()> {
        let login = login.clone().expect("No login passed");
        let client = self.state.pool.get().await.unwrap();
        let uuid = Uuid::parse_str(&login.id.expect("Login must contain user id")).unwrap();
        match user_service::authenticate_user(
            client,
            &uuid,
            login.password.expect("Login must contain user password"),
        ).await {
            Ok(res) => Ok(
                if res { 
                    let secret = std::env::var("JWT_SECRET").unwrap();
                    let secret = secret.as_bytes();
                    let token = Some(auth::create_token(&uuid, secret).unwrap());
                    LoginPostResponse::Status200(models::LoginPost200Response{token}) 
                }
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
        _: &Self::Claims,
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