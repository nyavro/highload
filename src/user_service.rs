use deadpool_postgres::{Object};
use crate::password_hash;
use uuid::Uuid;

#[derive(Debug)]
pub struct UserRegistration<'a> {
    pub first_name: &'a String,
    pub second_name: &'a String,
    pub birthdate: &'a chrono::naive::NaiveDate,
    pub biography: &'a String,
    pub city: &'a String,
    pub password: &'a String,
}

#[derive(Debug)]
pub struct UserRegistrationResult {
    pub user_id: Option<String>,
}

async fn check_if_user_exists(client: &Object, second_name: &String) -> Result<bool, String> {    
    let stmt = client.prepare_cached("SELECT 1 FROM users WHERE second_name=$1").await.unwrap();
    let rows = client.query(&stmt, &[second_name]).await.unwrap();
    if rows.is_empty() {
        Ok(true)
    } else {
        Err("User with second name already exists".to_string())        
    }   
}

pub async fn register_user<'a>(client: Object, req: UserRegistration<'a>) -> Result<UserRegistrationResult, String> {
    let second_name = req.second_name.clone();    
    match check_if_user_exists(&client, &second_name).await {
        Ok(_) => {
            let (_salt, password_hash) = password_hash::hash_password(
                req.password.clone()
            );
            let statement = client.prepare_cached("INSERT INTO users (first_name, second_name, birthdate, biography, city, pwd) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id").await.unwrap();
            let res = client.query_one(
                &statement, 
                &[
                    &req.first_name.clone(),
                    &req.second_name.clone(),
                    &req.birthdate,
                    &req.biography,
                    &req.city,
                    &password_hash
                ]).await.unwrap();
            let id: Uuid = res.get(0);
            Ok(UserRegistrationResult {
                user_id: Some(id.to_string())
            })
        },
        Err(e) => Err(e)
    }    
}

pub async fn authenticate_user(client: Object, id: &Uuid, password: String) -> Result<bool, String> {
    let st = client.prepare_cached("SELECT pwd FROM users WHERE id=$1").await.unwrap();
    let res = client.query_one(&st, &[&id]).await.unwrap();
    let hash: String = res.get(0);    
    Ok(password_hash::check_password(password, hash))
}