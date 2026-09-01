use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
use crate::{app_state::AppState, modules::{counter::repository::{CounterError, CounterRepository, CounterRepositoryImpl}, saga::repository::{SagaLogRepository, SagaLogRepositoryImpl, SagaLogError, SagaStatus}}};

#[derive(Error, Debug)]
pub enum SagaError {
    #[error("Counter error: {0}")]
    Counter(#[from] CounterError),
    #[error("Saga error: {0}")]
    Saga(#[from] SagaLogError),
    #[error("Orchestrator error: {0}")]
    Other(String),
}

pub struct SagaOrchestrator {
    counter_repo: CounterRepositoryImpl,
    saga_repo: SagaLogRepositoryImpl,
}

impl SagaOrchestrator {
    pub fn new(state: &AppState) -> Self {
        Self {
            counter_repo: CounterRepositoryImpl::new(Arc::clone(&state.redis_pool)),
            saga_repo: SagaLogRepositoryImpl::new(Arc::clone(&state.postgres_pool)),
        }
    }

    pub async fn inc(&self, user_id: &str) -> Result<i64, SagaError> {
        let (saga_id, _) = self.saga_repo.create_saga("MessageSend", user_id).await?;
        let result = self.counter_repo.increment(user_id).await;
        self.handle_result(saga_id, result).await
    }

    pub async fn dec(&self, user_id: &str) -> Result<i64, SagaError> {
        let (saga_id, _) = self.saga_repo.create_saga("MessageSend", user_id).await?;
        let result = self.counter_repo.decrement(user_id).await;        
        self.handle_result(saga_id, result).await
    }

    async fn handle_result(&self, saga_id: Uuid, result: Result<i64, CounterError>) -> Result<i64, SagaError> {
        match result {
            Ok(count) => {
                self.saga_repo.update_saga(saga_id, &SagaStatus::Completed, None).await?;
                self.saga_repo.update_saga_value(saga_id, count).await?;
                Ok(count)
            }
            Err(e) => {
                tracing::info!("Error: {:?}", e);
                self.saga_repo.update_saga(saga_id, &SagaStatus::Deferred, Some(format!("Redis error: {e}"))).await?;
                Err(SagaError::Counter(e))
            }
        }
    }
}