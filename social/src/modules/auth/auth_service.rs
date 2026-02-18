use uuid::Uuid;
use deadpool_postgres::{Object};
use crate::modules::auth::password_hash;

pub async fn authenticate_user(client: Object, id: &Uuid, password: &String) -> Result<bool, String> {    
    let res = client.query_one("SELECT pwd FROM users WHERE id=$1", &[&id]).await.unwrap();
    let hash: String = res.get(0);    
    Ok(password_hash::check_password(password, hash))
}