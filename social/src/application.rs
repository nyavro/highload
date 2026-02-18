use openapi::apis::{ApiAuthBasic, BasicAuthKind, ErrorHandler};
use async_trait::async_trait; 
use std::sync::Arc;
use crate::modules::auth::auth;
use crate::app_state::AppState;

#[derive(Clone)]
pub struct Application {
    pub state: Arc<AppState>, 
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

