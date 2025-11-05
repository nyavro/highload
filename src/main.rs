use log::{info};
use tokio_postgres::{Error};
use dotenv::dotenv;
use std::sync::Arc;
use app_state::AppState;
use openapi::apis::default::{Default, UserRegisterPostResponse, LoginPostResponse, UserGetIdGetResponse};
use openapi::apis::ErrorHandler;
use axum_extra::extract::{CookieJar, Host};
use axum::http::Method;
use async_trait::async_trait; 
use openapi::models::{self, User};

mod app_state;
mod migrations;

#[derive(Clone)]
struct Server {
    state: Arc<AppState>, 
}

impl AsRef<Server> for Server {
    fn as_ref(&self) -> &Server {
        &self
    }
}

impl Server {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl ErrorHandler for Server {}

#[async_trait]
impl Default for Server {

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
        _: &Option<models::UserRegisterPostRequest>
    ) -> Result<UserRegisterPostResponse, ()> {
        Ok(
            UserRegisterPostResponse::Status200(
                models::UserRegisterPost200Response {
                    user_id: Some("1234".to_string())
                }
            )
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    env_logger::init();    
    let app_state = Arc::new(AppState::init().await);
    migrations::run_migrations(app_state.clone()).await;
    let server = Server::new (app_state);
    let app = openapi::server::new(server);    
    let port = 3000;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    info!("Server is running on port {}", port);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
