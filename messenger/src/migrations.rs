use std::sync::Arc;
use crate::app_state::AppState;

mod embedded {
    refinery::embed_migrations!("./init/migrations");
}

pub async fn run_migrations(app_state: Arc<AppState>) {
    let mut client = app_state.get_master_client().await;
    tracing::info!("Running DB migrations...");
    let report = embedded::migrations::runner().run_async(&mut **client).await.unwrap();
    for migration in report.applied_migrations() {
        tracing::info!("Migration Applied - Name: {}, Version: {}", migration.name(), migration.version());
    }
    tracing::info!("Applied {} migrations", report.applied_migrations().len());  
}
