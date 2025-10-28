use log::{info, error};
use tokio_postgres::{NoTls, Error};
use dotenv::dotenv;
use axum::{routing::get, Router};

mod embedded {
    refinery::embed_migrations!("migrations");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    env_logger::init();    
    let (mut client, connection) = tokio_postgres::connect(
        "host=localhost user=pguser password=pgpassword dbname=highload",
        NoTls
    ).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Db connection error: {}", e);
        }
    });    
    info!("Running DB migrations...");
    let report = embedded::migrations::runner().run_async(&mut client).await.unwrap();
    for migration in report.applied_migrations() {
        info!("Migration Applied - Name: {}, Version: {}", migration.name(), migration.version());
    }
    info!("Applied {} migrations", report.applied_migrations().len());  
      
    let app = Router::new().route("/", get(|| async { "Basics)" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
