use async_trait::async_trait;
use deadpool::managed::{Manager, Metrics, RecycleError, RecycleResult};
use tarantool_rs::{Connection, ExecutorExt};

pub struct TarantoolManager {
    addr: String
}

impl TarantoolManager {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string()
        }
    }
}

impl Manager for TarantoolManager {
    type Type = Connection;
    type Error = tarantool_rs::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let addr = self.addr.clone();
        Connection::builder()
            .build(addr)
            .await
    }

    async fn recycle(&self, conn: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {        
        match conn.ping().await {
            Ok(_) => Ok(()),
            Err(e) => Err(RecycleError::Backend(e))
        }
    }
}