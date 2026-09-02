use std::{error::Error, sync::Arc, time::Duration};

use tokio::time::{MissedTickBehavior, interval};
use tracing::{info, warn};

use crate::{app_state::AppState, modules::{counter::repository::{CounterRepository, CounterRepositoryImpl}, saga::repository::{SagaLogRepository, SagaLogRepositoryImpl, SagaStatus}}};

pub async fn run_reconciliation_loop(state: Arc<AppState>, tick_interval: Duration) {
    info!("Reconciliation loop started (tick: {:?})", tick_interval);
    let mut ticker = interval(tick_interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        ticker.tick().await;
        if let Err(e) = run_once(Arc::clone(&state)).await {
            warn!("Reconciliation error: {:?}", e)
        }
    }
}

async fn run_once(state: Arc<AppState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let saga_repo = SagaLogRepositoryImpl::new(Arc::clone(&state.postgres_pool));
    let counter_repo = CounterRepositoryImpl::new(Arc::clone(&state.redis_pool));
    let deferred = saga_repo.get_by_status(&SagaStatus::Deferred).await?;
    if deferred.is_empty() {
        info!("No deferred tasks, skip");
        return Ok(())
    }
    info!("To reconcile {} saga records", deferred.len());
    for record in &deferred {
        match record.saga_type.as_str() {
            "MessageSend" => {
                match counter_repo.increment(&record.user_id).await {
                    Ok(count) => {
                        info!("Deferred MessageSend saga {} reconciled, count={}", record.saga_id, count);
                        saga_repo.update_saga(record.saga_id, &SagaStatus::Completed, Some("Reconciled".to_string())).await?;
                        saga_repo.update_saga_value(record.saga_id, count).await?;
                    },
                    Err(e) => {
                        warn!("Deferred MessageSend saga {} still failing: {:?}", record.saga_id, e);
                    }
                }
            },
            "DialogRead" => {
                match counter_repo.decrement(&record.user_id).await {
                    Ok(count) => {
                        info!("Deferred DialogRead saga {} reconciled, count={}", record.saga_id, count);
                        saga_repo.update_saga(record.saga_id, &SagaStatus::Completed, Some("Reconciled".to_string())).await?;
                        saga_repo.update_saga_value(record.saga_id, count).await?;
                    },
                    Err(e) => {
                        warn!("Defererred MessageSend saga {} still failing: {:?}", record.saga_id, e);
                    }
                }
            },
            other => {
                warn!("Unknown saga_type: {}", other);
            }
        }
    }
    Ok(())
}