use openapi::apis::post::{Post, PostCreatePostResponse};
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self};
use crate::modules::post::post_service;
use crate::modules::auth::auth;
use crate::Application;

#[async_trait]
impl Post for Application {
    type Claims = auth::Claims;

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
                        log::error!("Create post error: {:?}", e);
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
