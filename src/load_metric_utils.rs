use crate::app_state::AppState;
use csv::Writer;
use std::sync::Arc;
use deadpool_postgres::{Object};

pub async fn generate_load_data(app_state: Arc<AppState>) {
    let target_dir = "load-testing";
    let client = app_state.pool.get().await.unwrap();
    generate_user_ids_csv(&client, &target_dir).await;
    generate_firstname_prefixes_csv(&client, &target_dir).await;
    generate_lastname_prefixes_csv(&client, &target_dir).await;    
}

async fn generate_user_ids_csv(client: &Object, target_dir: &str) {
    let statement = client.prepare_cached("SELECT id FROM users LIMIT 1009").await.unwrap();            
    let mut wrtr = Writer::from_path(target_dir.to_owned() + "/user_ids.csv").unwrap();
    let res = client.query(&statement, &[]).await.unwrap();    
    for result in res {        
        let id: Option<uuid::Uuid> = result.get("id");
        wrtr.write_record(&[id.map(|t| t.to_string()).unwrap()]).unwrap();
    }
    wrtr.flush().unwrap();
}
async fn generate_firstname_prefixes_csv(client: &Object, target_dir: &str) {
    let statement = client.prepare_cached("SELECT DISTINCT SUBSTRING(Q.first_name, 0, 4) pref FROM (SELECT first_name, birthdate FROM users ORDER BY birthdate) Q LIMIT 997").await.unwrap();            
    let mut wrtr = Writer::from_path(target_dir.to_owned() + "/first_prefs.csv").unwrap();
    let res = client.query(&statement, &[]).await.unwrap();
    for result in res {        
        let first_pref: &str = result.get("pref");
        wrtr.write_record(&[&first_pref]).unwrap();
    }
    wrtr.flush().unwrap();        
}
async fn generate_lastname_prefixes_csv(client: &Object, target_dir: &str) {
    let statement = client.prepare_cached("SELECT DISTINCT SUBSTRING(Q.last_name, 0, 4) pref FROM (SELECT last_name, id FROM users ORDER BY id DESC) Q LIMIT 991").await.unwrap();            
    let mut wrtr = Writer::from_path(target_dir.to_owned() + "/last_prefs.csv").unwrap();
    let res = client.query(&statement, &[]).await.unwrap();
    for result in res {        
        let last_pref: &str = result.get("pref");
        wrtr.write_record(&[&last_pref]).unwrap();
    }
    wrtr.flush().unwrap();  
}