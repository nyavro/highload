use openapi::apis::default::{Default, LoginPostResponse, UserGetIdGetResponse, UserRegisterPostResponse, UserSearchGetResponse, FriendSetUserIdPutResponse, FriendDeleteUserIdPutResponse, PostCreatePostResponse};
use openapi::apis::{ApiAuthBasic, BasicAuthKind, ErrorHandler};
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self, User};
use std::sync::Arc;
use crate::app_state::AppState;
use crate::user_service;
use crate::friend_service;
use crate::post_service;
use uuid::Uuid;
use crate::auth;

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
        let auth_header = headers.get(axum::http::header::AUTHORIZATION)?;
        let auth_str = auth_header.to_str().ok()?;
        let token = auth_str.strip_prefix("Bearer ")?;
        auth::verify_token(token, &self.state.secret.as_bytes()).ok()
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
        let login_data = login.as_ref().ok_or(())?;        
        let uuid = match Uuid::parse_str(&login_data.id) {
            Ok(id) => id,
            Err(_) => return Ok(LoginPostResponse::Status400)
        };
        match user_service::authenticate_user(
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

    async fn user_get_id_get(
        &self,
        _: &Method,
        _: &Host,
        _: &CookieJar,
        _: &Self::Claims,
        path_params: &models::UserGetIdGetPathParams
    ) -> Result<UserGetIdGetResponse, ()> {
        let uuid = match Uuid::parse_str(&path_params.id) {
            Ok(id) => id,
            Err(_) => return Ok(UserGetIdGetResponse::Status400)
        };
        match user_service::get_user_by_id(
            self.state.get_replica_client().await, 
            uuid
        ).await {
            Ok(user) => Ok(UserGetIdGetResponse::Status200(to_user_dto(user))),
            Err(_) => Ok(UserGetIdGetResponse::Status400)
        }        
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
                                user_id: r.user_id.map(|t| t.to_string())
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
        let uuid = match Uuid::parse_str(&path_params.user_id) {
            Ok(id) => id,
            Err(_) => return Ok(FriendSetUserIdPutResponse::Status400)
        };
        match friend_service::add_friend(
            self.state.get_master_client().await, 
            claims.user_id, 
            uuid
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
        let cur_user_id = match Uuid::parse_str(&path_params.user_id) {
            Ok(id) => id,
            Err(_) => return Ok(FriendDeleteUserIdPutResponse::Status400)
        };        
        match friend_service::delete_friend(
            self.state.get_master_client().await, 
            claims.user_id, 
            cur_user_id,
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

    async fn post_create_post(
        &self,
        _: &Method,
        _: &Host,
        _: &CookieJar,
        claims: &Self::Claims,
        body: &Option<models::PostCreatePostRequest>,
    ) -> Result<PostCreatePostResponse, ()> {
        match body {
            Some(post) => {
                match post_service::create(self.state.get_master_client().await, claims.user_id, &post.text).await {
                    Ok(post_id) => Ok(PostCreatePostResponse::Status200(post_id.to_string())),
                    Err(e) => {
                        log::error!("Friendship end error: {:?}", e);
                        Ok(PostCreatePostResponse::Status500 {
                            body: models::LoginPost500Response { 
                                message: "Internal Server Error".to_string(),
                                request_id: None,
                                code: None
                            },
                            retry_after: None,
                        })
                    }
                }                
            },
            None => Ok(PostCreatePostResponse::Status400)
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