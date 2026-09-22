use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::registry::{Registry};
use prometheus_client::metrics::{counter::Counter, gauge::Gauge, histogram::Histogram};
use once_cell::sync::Lazy;
use prometheus_client::metrics::family::Family;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct MetricLabels {
    pub handler: String,
    pub method: String,
    pub status_code: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct DurationLabels {
    pub handler: String,
    pub method: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct DialogLabels {
    pub dialog_id: String,    
}

static REQUESTS_TOTAL: Lazy<Family<MetricLabels, Counter<u64>>> = Lazy::new(|| {Family::default()});
static ERRORS_TOTAL: Lazy<Family<MetricLabels, Counter<u64>>> = Lazy::new(|| {Family::default()});
static MESSAGES_SENT_TOTAL: Lazy<Family<MetricLabels, Counter<u64>>> = Lazy::new(|| {Family::default()});
static MESSAGES_RECEIVED_TOTAL: Lazy<Family<MetricLabels, Counter<u64>>> = Lazy::new(|| {Family::default()});
static REQUEST_DURATION: Lazy<Family<DurationLabels, Histogram>> = Lazy::new(|| {
    Family::new_with_constructor(|| {
        Histogram::new(vec![
            0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0
        ].into_iter())
    })
});
static ACTIVE_CONNECTIONS: Lazy<Gauge> = Lazy::new(Gauge::default);
static ACTIVE_DIALOGS: Lazy<Gauge> = Lazy::new(Gauge::default);
static MESSAGE_SEND_DURATION: Lazy<Histogram> = Lazy::new(|| {    
    Histogram::new(vec![
        0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0
    ].into_iter())    
});
static POSTGRES_POOL_SIZE: Lazy<Gauge> = Lazy::new(Gauge::default);
static TARANTOOL_POOL_SIZE: Lazy<Gauge> = Lazy::new(Gauge::default);


static REGISTRY: Lazy<Registry> = Lazy::new(|| {
    let mut registry = Registry::default();    
    registry.register("requests_total", "Total number of HTTP requests(RED metric)", REQUESTS_TOTAL.clone());
    registry.register("errors_total", "Total number of HTTP errors with status >= 400 (RED metric)", ERRORS_TOTAL.clone());
    registry.register("request_duration_seconds", "HTTP request duration in seconds (RED metric)", REQUEST_DURATION.clone()); 
    registry.register("active_connections", "Number of currently active HTTP connections", ACTIVE_CONNECTIONS.clone());
    registry.register("messages_sent_total", "Total number of messages sent by users", MESSAGES_SENT_TOTAL.clone());
    registry.register("messages_received_total", "Total number of messages received by users", MESSAGES_RECEIVED_TOTAL.clone());
    registry.register("active_dialogs", "Number of active dialogs", ACTIVE_DIALOGS.clone());
    registry.register("message_send_duration_seconds", "Time taken to send a message", MESSAGE_SEND_DURATION.clone());
    registry.register("postgres_pool_size", "Current size of the PostgreSQL connection pool", POSTGRES_POOL_SIZE.clone());
    registry.register("tarantool_pool_size", "Current size of the Tarantool connection pool", TARANTOOL_POOL_SIZE.clone());
    registry
});

pub fn inc_requests(handler: &str, method: &str, status_code: u16) {
    let labels = MetricLabels {
        handler: handler.to_string(),
        method: method.to_string(),
        status_code,
    };
    REQUESTS_TOTAL.get_or_create(&labels).inc();
}

pub fn inc_errors(handler: &str, method: &str, status_code: u16) {
    let labels = MetricLabels {
        handler: handler.to_string(),
        method: method.to_string(),
        status_code,
    };    
    ERRORS_TOTAL.get_or_create(&labels).inc();
}

pub fn observe_request_duration(handler: &str, method: &str, duration: f64) {
    let labels = DurationLabels {
        handler: handler.to_string(),
        method: method.to_string(),
    };
    REQUEST_DURATION.get_or_create(&labels).observe(duration);
}

pub fn set_active_connections(count: i64) {
    ACTIVE_CONNECTIONS.set(count);
}

pub fn inc_messages_sent(handler: &str, method: &str, status_code: u16) {
    let labels = MetricLabels {
        handler: handler.to_string(),
        method: method.to_string(),
        status_code,
    };
    MESSAGES_SENT_TOTAL.get_or_create(&labels).inc();
}

pub fn inc_messages_received(handler: &str, method: &str, status_code: u16) {
    let labels = MetricLabels {
        handler: handler.to_string(),
        method: method.to_string(),
        status_code,
    };
    MESSAGES_RECEIVED_TOTAL.get_or_create(&labels).inc();
}

pub fn set_active_dialogs(count: i64) {
    ACTIVE_DIALOGS.set(count);
}

pub fn observe_message_send_duration(duration: f64) {    
    MESSAGE_SEND_DURATION.observe(duration);
}

pub fn set_postgres_pool_size(size: i64) {
    POSTGRES_POOL_SIZE.set(size);
}

pub fn set_tarantool_pool_size(size: i64) {
    TARANTOOL_POOL_SIZE.set(size);
}

pub async fn metrics_handler() -> axum::response::Response {
    let mut buffer = String::new();
    prometheus_client::encoding::text::encode(&mut buffer, &REGISTRY).unwrap();
    axum::response::Response::builder()
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(axum::body::Body::from(buffer))
        .unwrap()
}