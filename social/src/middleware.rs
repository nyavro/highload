use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use http::Extensions;
use reqwest::header::{HeaderValue, AUTHORIZATION};

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub request_id: String,
    pub jwt_token: Option<String>,
}

tokio::task_local! {
    pub static CURRENT_CONTEXT: RequestContext;
}

pub struct PropagateTracingIdMiddleware;

#[async_trait::async_trait]
impl Middleware for PropagateTracingIdMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {        
        if let Ok(ctx) = CURRENT_CONTEXT.try_with(|ctx| ctx.clone()) {            
            if let Ok(id_val) = HeaderValue::from_str(&ctx.request_id) {
                req.headers_mut().insert("x-request-id", id_val);
            }
            if let Some(token) = ctx.jwt_token {
                if let Ok(token_val) = HeaderValue::from_str(&token) {
                    req.headers_mut().insert(AUTHORIZATION, token_val);
                }
            }
        } else {         
            req.headers_mut().insert("x-request-id", HeaderValue::from_static("internal-job"));
        }
        next.run(req, extensions).await
    }
}