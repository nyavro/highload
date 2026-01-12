use openapi::apis::friend::{Friend, FriendSetUserIdPutResponse, FriendDeleteUserIdPutResponse};
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self};
use crate::modules::friend::friend_service;
use uuid::Uuid;
use crate::modules::auth::auth;
use crate::Application;

#[async_trait]
impl Friend for Application {
    type Claims = auth::Claims;

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
}
