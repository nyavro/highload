use openapi::apis::post::{Post, PostPostResponse, PostPutResponse, PostIdDeleteResponse};
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self};
use crate::modules::post::post_service;
use crate::modules::auth::auth;
use crate::Application;
use uuid::Uuid;

#[async_trait]
impl Post for Application {
    type Claims = auth::Claims;

    async fn post_post(
        &self,
        _: &Method,
        _: &Host,
        _: &CookieJar,
        claims: &Self::Claims,
        body: &Option<models::PostPostRequest>,
    ) -> Result<PostPostResponse, ()> {
        match body {
            Some(post) => {
                match post_service::create(self.state.get_master_client().await, claims.user_id, &post.text).await {
                    Ok(post_id) => Ok(PostPostResponse::Status200(post_id.to_string())),
                    Err(e) => {
                        log::error!("Create post error: {:?}", e);
                        Ok(PostPostResponse::Status500 {
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
            None => Ok(PostPostResponse::Status400)
        }
    }

    async fn post_put(
        &self,    
        _: &Method,
        _: &Host,
        _: &CookieJar,
        claims: &Self::Claims,
        body: &Option<models::PostPutRequest>,
    ) -> Result<PostPutResponse, ()> {
        match body {
            Some(post) => {
                let post_id = match Uuid::parse_str(&post.id) {
                    Ok(id) => id,
                    Err(_) => return Ok(PostPutResponse::Status400)
                };
                match post_service::update(self.state.get_master_client().await, claims.user_id, post_id, &post.text).await {
                    Ok(()) => Ok(PostPutResponse::Status200),
                    Err(e) => {
                        log::error!("Create post error: {:?}", e);
                        Ok(PostPutResponse::Status500 {
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
            None => Ok(PostPutResponse::Status400)
        }
    }

    async fn post_id_delete(
        &self,    
        _: &Method,
        _: &Host,
        _: &CookieJar,
        claims: &Self::Claims,
        path_params: &models::PostIdDeletePathParams
    ) -> Result<PostIdDeleteResponse, ()> {
        let post_id = match Uuid::parse_str(&path_params.id) {
            Ok(id) => id,
            Err(_) => return Ok(PostIdDeleteResponse::Status400)
        };
        match post_service::delete(self.state.get_master_client().await, claims.user_id, post_id).await {
            Ok(()) => Ok(PostIdDeleteResponse::Status200),
            Err(e) => {
                log::error!("Create post error: {:?}", e);
                Ok(PostIdDeleteResponse::Status500 {
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
