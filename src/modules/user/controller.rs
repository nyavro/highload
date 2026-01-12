use openapi::apis::user::{User, UserGetIdGetResponse, UserRegisterPostResponse, UserSearchGetResponse};
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self};
use crate::modules::user::user_service;
use uuid::Uuid;
use crate::modules::auth::auth;
use crate::Application;

#[async_trait]
impl User for Application {
    type Claims = auth::Claims;
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
}

fn to_user_dtos(users: Vec<user_service::User>) -> Vec<openapi::models::User> {
    users.into_iter().map(to_user_dto).collect()
}

fn to_user_dto(user: user_service::User) -> openapi::models::User {
    openapi::models::User {
        id: user.id.map(|t| t.to_string()),
        first_name: user.first_name,
        last_name: user.last_name,
        birthdate: user.birthdate,
        biography: user.biography,
        city: user.city,
    }    
}