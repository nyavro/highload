use axum::response::{ IntoResponse, Response };
use prometheus_client::registry::{Registry};
use prometheus_client::metrics::{counter::Counter, gauge::Gauge, histogram::Histogram};
use once_cell::sync::Lazy;
use prometheus_client::encoding::text::encode;
use std::sync::Mutex;

static REQUESTS_TOTAL: Lazy<Counter<u64>> = Lazy::new(Counter::default);
static ERRORS_TOTAL: Lazy<Counter<u64>> = Lazy::new(Counter::default);
static ACTIVE_CONNECTIONS: Lazy<Gauge> = Lazy::new(Gauge::default);
static REQUEST_DURATION: Lazy<Histogram> = Lazy::new(|| {
    Histogram::new(vec![
        0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0
    ].into_iter())
});

static REGISTRY: Lazy<Mutex<Registry>> = Lazy::new(|| {
    let mut registry = Registry::default();    
    registry.register("requests_total", "Total number of HTTP requests", REQUESTS_TOTAL.clone());
    registry.register("errors_total", "Total number of HTTP errors", ERRORS_TOTAL.clone());
    registry.register("active_connections", "Number of currently active connections", ACTIVE_CONNECTIONS.clone());
    registry.register("request_duration_seconds", "HTTP request duration in seconds", REQUEST_DURATION.clone()); 
    Mutex::new(registry)
});

pub fn inc_requests(count: u64) {
    REQUESTS_TOTAL.inc_by(count);
}

pub fn inc_errors(count: u64) {
    ERRORS_TOTAL.inc_by(count);
}

pub fn set_active_connections(count: i64) {
    ACTIVE_CONNECTIONS.set(count);
}

pub fn observe_duration(duration: f64) {
    REQUEST_DURATION.observe(duration);
}

fn encode_metrics() -> String {
    let mut buffer = String::new();
    let registry = REGISTRY.lock().unwrap();
    encode(&mut buffer, &registry).unwrap();
    buffer
}

pub async fn metrics_handler() -> impl IntoResponse {
    Response::builder()
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(encode_metrics())
        .unwrap()
}