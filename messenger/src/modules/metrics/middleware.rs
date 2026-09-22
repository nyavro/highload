use crate::modules::metrics::handler;

fn extract_handler(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut result = String::new();
    for segment in segments {
        result.push('/');
        if segment.parse::<u64>().is_ok() || is_uuid(segment) || segment == "v1" || segment == "v2" {
            result.push_str("{id}");
        } else {
            result.push_str(segment);
        }
    }
    result
}

fn is_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    for (i, part) in parts.iter().enumerate() {
        let expected_length = match i {
            0 => 8,
            1 | 2 | 3 => 4,
            4 => 12,
            _ => unreachable!(),
        };
        if part.len() != expected_length || !part.chars().all(|c| c.is_digit(16)) {
            return false;
        }
    }
    true
}

pub async fn metrics_middleware(req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next) -> impl axum::response::IntoResponse {    
    let start_time = std::time::Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let handler = extract_handler(&path);
    handler::inc_requests(&handler, &method, 0);
    let response = next.run(req).await;
    let status = response.status().as_u16();
    let duration = start_time.elapsed().as_secs_f64();
    handler::observe_request_duration(&handler, &method, duration);
    if status >= 400 {
        handler::inc_errors(&handler, &method, status);
    }
    handler::set_active_connections(1);
    tracing::info!(
        method = %method,
        path = %path,
        handler = %handler,
        status = %status,
        duration_sec = %duration,
        "HTTP request completed"
    );
    response
}