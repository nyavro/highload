use crate::modules::metrics::handler;

pub async fn metrics_middleware(req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next) -> impl axum::response::IntoResponse {    
    let start_time = std::time::Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    handler::inc_requests(1);
    handler::set_active_connections(1);
    
    let response = next.run(req).await;
    let status = response.status().as_u16();
    let duration = start_time.elapsed().as_secs_f64();

    handler::observe_duration(duration);

    if status >= 400 {
        handler::inc_errors(1);
    }
    let status_str = status.to_string();
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %status_str,
        duration = %duration,
        "HTTP request"
    );
    response
}