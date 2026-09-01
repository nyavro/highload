use std::{fmt::Display, sync::Arc};

use async_trait::async_trait;
use deadpool_postgres::Pool;
use serde::Serialize;
use thiserror::Error;
use deadpool::managed::PoolError;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum SagaLogError {
    #[error("Pool error: {0}")]
    Pool(#[from] PoolError<tokio_postgres::Error>),
    #[error("Pool error: {0}")]
    Database(#[from] tokio_postgres::Error),
    #[error("Saga error: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, PartialEq)] 
pub enum SagaStatus {
    Started, Completed, Compensated, Deferred
}

impl Display for SagaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SagaStatus::Compensated => write!(f, "COMPENSATED"),
            SagaStatus::Completed => write!(f, "COMPLETED"),
            SagaStatus::Deferred => write!(f, "DEFERRED"),
            SagaStatus::Started => write!(f, "STARTED"),
        }
    }
}

#[derive(Debug, Clone, Serialize)] 
pub struct SagaRecord {
    pub saga_id: Uuid,
    pub saga_type: String,
    pub user_id: String,
    pub status: String,
    pub value: Option<i64>,
    pub compensation: Option<String>,
    pub created_at: String,
}


#[async_trait]
pub trait SagaLogRepository {
    async fn create_saga(&self, saga_type: &str, user_id: &str) -> Result<(Uuid, SagaRecord), SagaLogError>;
    async fn update_saga(&self, saga_id: Uuid, new_status: &SagaStatus, compensation: Option<String>) -> Result<(), SagaLogError>;
    async fn update_saga_value(&self, saga_id: Uuid, value: i64) -> Result<(), SagaLogError>;
    async fn get_by_status(&self, status: &SagaStatus) -> Result<Vec<SagaRecord>, SagaLogError>;
    async fn get_by_id(&self, saga_id: Uuid) -> Result<Option<SagaRecord>, SagaLogError>;
}

pub struct SagaLogRepositoryImpl {
    pub pool: Arc<Pool>,
}

impl SagaLogRepositoryImpl {
    pub fn new(pool: Arc<Pool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SagaLogRepository for SagaLogRepositoryImpl {
    async fn create_saga(&self, saga_type: &str, user_id: &str) -> Result<(Uuid, SagaRecord), SagaLogError> {
        let saga_id = Uuid::new_v4();
        let client = self.pool.get().await?;        
        client.execute(
            "INSERT INTO saga_log(id, saga_type, user_id, status) VALUES ($1, $2, $3, $4)",
            &[
                &saga_id,
                &saga_type,
                &user_id,
                &SagaStatus::Started.to_string()
            ]
        ).await?;
        Ok((
            saga_id,
            SagaRecord {
                saga_id,
                saga_type: saga_type.to_string(),
                user_id: user_id.to_string(),
                status: SagaStatus::Started.to_string(),
                value: None,
                compensation: None,
                created_at: chrono::Utc::now().to_rfc3339()
            }
        ))
    }

    async fn update_saga(&self, saga_id: Uuid, new_status: &SagaStatus, compensation: Option<String>) -> Result<(), SagaLogError> {        
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE saga_log SET status = $1, compensation = $2 WHERE id = $3",
            &[
                &new_status.to_string(),
                &compensation,
                &saga_id
            ]
        ).await?;
        Ok(())
    }

    async fn update_saga_value(&self, saga_id: Uuid, value: i64) -> Result<(), SagaLogError> {        
        let client = self.pool.get().await?;
        client.execute(
            "UPDATE saga_log SET value = $1 WHERE id = $2",
            &[
                &value,
                &saga_id
            ]
        ).await?;
        Ok(())
    }

    async fn get_by_status(&self, status: &SagaStatus) -> Result<Vec<SagaRecord>, SagaLogError> {
        let client = self.pool.get().await?;
        let rows = client.query(
            "SELECT id, saga_type, user_id, status, value, compensation, created_at FROM saga_log WHERE status = $1 ORDER BY created_at ASC",
            &[&status.to_string()]
        ).await?;
        let records: Vec<SagaRecord> = rows.iter().map(|row| SagaRecord {
            saga_id: row.get("id"),
            saga_type: row.get("saga_type"),
            user_id: row.get("user_id"),
            status: row.get("status"),
            value: row.get("value"),
            compensation: row.get("compensation"),
            created_at: row.get("created_at"),
        }).collect();
        Ok(records)
    }

    async fn get_by_id(&self, saga_id: Uuid) -> Result<Option<SagaRecord>, SagaLogError> {
        let client = self.pool.get().await?;
        let row = client.query_opt(
            "SELECT saga_id, saga_type, user_id, status, value, compensation, created_at FROM saga_log WHERE id = $1",
            &[&saga_id]
        ).await?;
        Ok(
            row.map(|row| SagaRecord {
                saga_id: row.get("id"),
                saga_type: row.get("saga_type"),
                user_id: row.get("user_id"),
                status: row.get("status"),
                value: row.get("value"),
                compensation: row.get("compensation"),
                created_at: row.get("created_at"),
            })
        )
    }
}