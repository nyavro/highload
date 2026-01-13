use openapi::apis::post::{Post, PostPostResponse, PostIdGetResponse, PostPutResponse, PostIdDeleteResponse, PostFeedGetResponse};
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

    async fn post_id_get(
        &self,    
        _: &Method,
        _: &Host,
        _: &CookieJar,
        path_params: &models::PostIdGetPathParams,
    ) -> Result<PostIdGetResponse, ()> {
        let post_id = match Uuid::parse_str(&path_params.id) {
            Ok(id) => id,
            Err(_) => return Ok(PostIdGetResponse::Status400)
        };
        match post_service::get(self.state.get_replica_client().await, post_id).await {
            Ok(post) => Ok(PostIdGetResponse::Status200(to_post_dto(post))),
            Err(e) => {
                log::error!("Get post error: {:?}", e);
                Ok(PostIdGetResponse::Status500 {
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
                        log::error!("Update post error: {:?}", e);
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
                log::error!("Delete post error: {:?}", e);
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

    async fn post_feed_get(
        &self,    
        _: &Method,
        _: &Host,
        _: &CookieJar,
        claims: &Self::Claims,
        query_params: &models::PostFeedGetQueryParams,
    ) -> Result<PostFeedGetResponse, ()> {
        match post_service::feed(self.state.get_master_client().await, claims.user_id, query_params.limit, query_params.offset).await {
            Ok(posts) => Ok(PostFeedGetResponse::Status200(to_post_dtos(posts))),
            Err(e) => {
                log::error!("Feed posts error: {:?}", e);
                Ok(PostFeedGetResponse::Status500 {
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

fn to_post_dto(post: post_service::Post) -> openapi::models::Post {
    openapi::models::Post {
        id: post.id.to_string(),
        text: post.text,
        author_user_id: post.author_user_id.to_string()
    }    
}

fn to_post_dtos(posts: Vec<post_service::Post>) -> Vec<openapi::models::Post> {
    posts.into_iter().map(to_post_dto).collect()
}