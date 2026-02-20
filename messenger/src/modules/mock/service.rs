use crate::app_state::AppState;
use std::sync::Arc;
use std::fs::read_to_string;
use rand::RngExt;
use uuid::Uuid;
use log::info;

pub async fn generate_messages(app_state: Arc<AppState>) {
    let mut uuids: Vec<Uuid> = vec!();
    for line in read_to_string("load-testing/user_ids.csv").unwrap().lines() {    
        uuids.push(Uuid::parse_str(line).unwrap());
    }
    let mut rng = rand::rng();
    for line in read_to_string("load-testing/posts.txt").unwrap().lines() {                    
        let from = uuids.get(rng.random_range(0..uuids.len())).unwrap().clone();
        let to = uuids.get(rng.random_range(0..uuids.len())).unwrap().clone();
        if let Ok(id) = app_state.dialog_service.send_message(from, to, &line.to_string()).await {
            info!("Created message id: {}", id);
        }
    }
}
