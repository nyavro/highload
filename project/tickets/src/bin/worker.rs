use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::Message;
use deadpool_postgres::{Config, Pool, Runtime};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use tracing::{error, info};
use std::env;
use std::time::Duration;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderPaidEvent {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub seat_id: String,
    pub reservation_id: String,
    pub amount_cents: i32,
    pub ticket_code: String,
    pub timestamp_ms: i64,
}

fn init_tracing() {
    tracing_subscriber::fmt()        
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into())
        )        
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .init(); 
}

async fn init_postgres_pool() -> Pool {
    let mut cfg = Config::new();
    cfg.user = Some(env::var("POSTGRES_USER").unwrap_or_else(|_| "pguser".to_string()));
    cfg.password = Some(env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "pgpassword".to_string()));
    cfg.dbname = Some(env::var("POSTGRES_DB_NAME").unwrap_or_else(|_| "tickets".to_string()));
    cfg.host = Some(env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string()));
    cfg.port = Some(env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string()).parse::<u16>().unwrap());

    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create postgres pool")
}

fn init_kafka_consumer() -> StreamConsumer {
    let kafka_brokers = env::var("KAFKA_BOOTSTRAP_SERVERS").unwrap_or_else(|_| "localhost:9092".to_string());
    ClientConfig::new()
        .set("bootstrap.servers", &kafka_brokers)
        .set("group.id", "tickets-worker-group") 
        .set("enable.auto.commit", "false") 
        .set("auto.offset.reset", "earliest") 
        .create()
        .expect("Failed to create Kafka consumer")
}

async fn save_ticket_to_db(pg_pool: &Pool, event: &OrderPaidEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = pg_pool.get().await?;
    let tx = client.transaction().await?;
    let insert_order_query = "
        INSERT INTO orders (id, user_id, event_id, status, amount_cents)
        VALUES ($1, $2, $3, $4, $5);
    ";
    tx.execute(
        insert_order_query,
        &[&event.order_id, &event.user_id, &event.event_id, &"paid", &event.amount_cents],
    ).await?;
    let seat_uuid = Uuid::parse_str(&event.seat_id)?;
    let ticket_id = Uuid::new_v4();

    let insert_ticket_query = "
        INSERT INTO tickets (id, order_id, event_id, seat_id, ticket_code)
        VALUES ($1, $2, $3, $4, $5);
    ";
    tx.execute(
        insert_ticket_query,
        &[&ticket_id, &event.order_id, &event.event_id, &seat_uuid, &event.ticket_code],
    ).await?;

    tx.commit().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();    
    init_tracing();     
    
    info!("Running Background Worker for tickets processing...");

    let pg_pool = init_postgres_pool().await;    
    let consumer: StreamConsumer = init_kafka_consumer();
    
    let topic_name = std::env::var("KAFKA_TICKETS_BILLING_TOPIC").unwrap_or_else(|_| "tickets-billing".to_string());
    consumer.subscribe(&[topic_name.as_str()]).expect("Failed to subscribe to topic");
    info!("Subscribed to topic: '{}'", topic_name);
    
    loop {
        match consumer.recv().await {
            Err(e) => println!("Kafka read error: {}", e),
            Ok(msg) => {                
                if let Some(payload) = msg.payload() {
                    if let Ok(payload) = std::str::from_utf8(payload) {
                        match serde_json::from_str::<OrderPaidEvent>(payload) {
                            Ok(event) => {
                                info!("Order paid event: {}. Persisting...", event.order_id);                                                        
                                match save_ticket_to_db(&pg_pool, &event).await {
                                    Ok(_) => {
                                        info!("Order {} successfully saved to DB", event.order_id);
                                        let _ = consumer.commit_message(&msg, CommitMode::Async);
                                    }
                                    Err(db_err) => {
                                        error!("Critical error {}: {}. Message not saved", event.order_id, db_err);
                                        tokio::time::sleep(Duration::from_secs(1)).await;
                                    }
                                }
                            }
                            Err(parse_err) => {
                                println!("Failed to parse JSON: {}. Skipping message.", parse_err);                            
                                let _ = consumer.commit_message(&msg, CommitMode::Async);
                            }
                        }
                    }
                }
            }
        }
    }
}
