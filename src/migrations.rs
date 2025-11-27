use std::sync::Arc;
use crate::app_state::AppState;
use log::info;

mod embedded {
    refinery::embed_migrations!("./migrations");
}

pub async fn run_migrations(app_state: Arc<AppState>) {
    let mut client = app_state.pool.get().await.unwrap();
    info!("Running DB migrations...");
    let report = embedded::migrations::runner().run_async(&mut **client).await.unwrap();
    for migration in report.applied_migrations() {
        info!("Migration Applied - Name: {}, Version: {}", migration.name(), migration.version());
    }
    info!("Applied {} migrations", report.applied_migrations().len());  
}
