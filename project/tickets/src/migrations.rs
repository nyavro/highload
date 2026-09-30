use std::sync::Arc;
use deadpool_postgres::Pool;

mod embedded {
    refinery::embed_migrations!("./migrations");
}

pub async fn run_migrations(pool: Arc<Pool>) {
    let mut client = pool.get().await.unwrap();
    tracing::info!("Running DB migrations...");
    let report = embedded::migrations::runner().run_async(&mut **client).await.unwrap();
    for migration in report.applied_migrations() {
        tracing::info!("Migration Applied - Name: {}, Version: {}", migration.name(), migration.version());
    }
    tracing::info!("Applied {} migrations", report.applied_migrations().len());  
}
