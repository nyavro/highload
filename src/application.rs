use openapi::apis::default::{Default, LoginPostResponse, UserGetIdGetResponse, UserRegisterPostResponse, UserSearchGetResponse, FriendSetUserIdPutResponse, FriendDeleteUserIdPutResponse};
use openapi::apis::{ApiAuthBasic, BasicAuthKind, ErrorHandler};
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self, User};
use std::panic::PanicHookInfo;
use std::sync::Arc;
use crate::app_state::AppState;
use crate::user_service;
use crate::friend_service;
use uuid::Uuid;
use crate::auth;
use axum::response::IntoResponse;
use axum::{http::{StatusCode}};
use log::info;

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
    async fn extract_claims_from_auth_header(&self, _kind: BasicAuthKind, headers: &axum::http::header::HeaderMap, _key: &str) -> Option<Self::Claims> {                                
        let token = headers.get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {(StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header").into_response()}).ok()
            .and_then(|value| value.strip_prefix("Bearer "))                
            .ok_or_else(|| {
                (StatusCode::UNAUTHORIZED, "Invalid authorization scheme, expected Bearer").into_response()
            })
            .unwrap();
        info!("{:?}", token);        
        Some(
            auth::verify_token(token, &self.state.secret.as_bytes())
                .map_err(|_| {
                    (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response()
                })
                .unwrap()
        )
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
        let uuid = Uuid::parse_str(&login.id).unwrap();
        match user_service::authenticate_user(
            self.state.get_master_client().await,
            &uuid,
            login.password,
        ).await {
            Ok(res) => Ok(
                if res { 
                    let secret = std::env::var("JWT_SECRET").unwrap();
                    let secret = secret.as_bytes();
                    let token = Some(auth::create_token(&uuid, secret, self.state.jwt_token_ttl_minutes).unwrap());
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
        let user = user_service::get_user_by_id(
            self.state.get_replica_client().await, 
            Uuid::parse_str(&path_params.id).unwrap()
        ).await.unwrap();        
        Ok(
            UserGetIdGetResponse::Status200(to_user_dto(user))
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
                    let res = user_service::register_user(
                        self.state.get_master_client().await,
                        user_service::UserRegistration {
                            first_name: &req.first_name,
                            last_name: &req.last_name,
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

    async fn user_search_get(
        &self,
        _: &Method,
        _: &Host,
        _: &CookieJar,
        _: &Self::Claims,
        search_request: &models::UserSearchGetQueryParams,        
        ) -> Result<UserSearchGetResponse, ()> {
        Ok(
            UserSearchGetResponse::Status200(
                to_user_dtos(
                    user_service::search_by_first_and_last_name(
                        self.state.get_replica_client().await, 
                        &search_request.first_name, 
                        &search_request.last_name
                    ).await
                )
            )
        )
    }

    async fn friend_set_user_id_put(
        &self,    
        _: &Method,
        _: &Host,
        _: &CookieJar,
        claims: &Self::Claims,
        path_params: &models::FriendSetUserIdPutPathParams,
    ) -> Result<FriendSetUserIdPutResponse, ()> {
        match friend_service::add_friend(
            self.state.get_master_client().await, 
            claims.user_id, 
            Uuid::parse_str(&path_params.user_id).unwrap()
        ).await {
            Ok(friend_service::FriendshipCreateResult::Accepted) => Ok(FriendSetUserIdPutResponse::Status200),
            Ok(friend_service::FriendshipCreateResult::RequestSent) => Ok(FriendSetUserIdPutResponse::Status200),
            Ok(friend_service::FriendshipCreateResult::AlreadyExists) => Ok(FriendSetUserIdPutResponse::Status400),             
            Err(e) => {
                log::error!("Friendship create error: {:?}", e);
                Ok(FriendSetUserIdPutResponse::Status500 {
                    body: models::LoginPost500Response { 
                        message: "Internal Server Error".to_string(),
                        request_id: None,
                        code: None
                    },
                    retry_after: None,
                })
            }
        }        
    }

    async fn friend_delete_user_id_put(
        &self,        
        _: &Method,
        _: &Host,
        _: &CookieJar,
        claims: &Self::Claims,
      path_params: &models::FriendDeleteUserIdPutPathParams,
    ) -> Result<FriendDeleteUserIdPutResponse, ()> {
        match friend_service::delete_friend(
            self.state.get_master_client().await, 
            claims.user_id, 
            Uuid::parse_str(&path_params.user_id).unwrap(),
            false
        ).await {
            Ok(friend_service::FriendshipEndResult::Subscribed) => Ok(FriendDeleteUserIdPutResponse::Status200),
            Ok(friend_service::FriendshipEndResult::Blocked) => Ok(FriendDeleteUserIdPutResponse::Status200),
            Ok(friend_service::FriendshipEndResult::NotInFriendship) => Ok(FriendDeleteUserIdPutResponse::Status400),             
            Err(e) => {
                log::error!("Friendship end error: {:?}", e);
                Ok(FriendDeleteUserIdPutResponse::Status500 {
                    body: models::LoginPost500Response { 
                        message: "Internal Server Error".to_string(),
                        request_id: None,
                        code: None
                    },
                    retry_after: None,
                })
            }
        }
    }
}

fn to_user_dtos(users: Vec<user_service::User>) -> Vec<User> {
    users.into_iter().map(to_user_dto).collect()
}

fn to_user_dto(user: user_service::User) -> User {
    User {
        id: user.id,
        first_name: user.first_name,
        last_name: user.last_name,
        birthdate: user.birthdate,
        biography: user.biography,
        city: user.city,
    }    
}