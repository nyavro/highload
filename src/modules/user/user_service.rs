use chrono::NaiveDate;
use deadpool_postgres::{Object};
use uuid::Uuid;
use crate::modules::auth::password_hash;

#[derive(Debug)]
pub struct UserRegistration<'a> {
    pub first_name: &'a String,
    pub last_name: &'a String,
    pub birthdate: &'a chrono::naive::NaiveDate,
    pub biography: &'a String,
    pub city: &'a String,
    pub password: &'a String,
}

#[derive(Debug)]
pub struct UserRegistrationResult {
    pub user_id: Option<uuid::Uuid>,
}

#[derive(Debug)]
pub struct User {
    pub id: Option<String>,            
    pub first_name: String,
    pub last_name: String,
    pub birthdate: chrono::naive::NaiveDate,
    pub biography: Option<String>,
    pub city: String,
}

async fn check_if_user_exists(client: &Object, last_name: &String) -> Result<bool, String> {        
    let rows = client.query("SELECT 1 FROM users WHERE last_name=$1", &[last_name]).await.unwrap();
    if rows.is_empty() {
        Ok(true)
    } else {
        Err("User with last name already exists".to_string())        
    }   
}

pub async fn register_user<'a>(client: Object, req: UserRegistration<'a>) -> Result<UserRegistrationResult, String> {
    let last_name = req.last_name.clone();    
    match check_if_user_exists(&client, &last_name).await {
        Ok(_) => {
            let (_salt, password_hash) = password_hash::hash_password(
                req.password.clone()
            );            
            let res = client.query_one(
                "INSERT INTO users (first_name, last_name, birthdate, biography, city, pwd) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id", 
                &[
                    &req.first_name.clone(),
                    &req.last_name.clone(),
                    &req.birthdate,
                    &req.biography,
                    &req.city,
                    &password_hash
                ]).await.unwrap();
            let id: Uuid = res.get(0);
            Ok(UserRegistrationResult {
                user_id: Some(id)
            })
        },
        Err(e) => Err(e)
    }    
}

pub async fn get_user_by_id(client: Object, id: Uuid) -> Result<User, String> {    
    let row = client.query_one("SELECT first_name, last_name, birthdate, biography, city FROM users WHERE id=$1", &[&id]).await.unwrap();
    let first_name: String = row.get(0);            
    let last_name: String = row.get(1);            
    let birthdate: NaiveDate = row.get(2);            
    let biography: Option<String> = row.get(3);            
    let city: String = row.get(4);   
    Ok(
        User{
            id: Some(id.to_string()),
            first_name,
            last_name,
            birthdate,
            biography,
            city
        }
    )
}

pub async fn search_by_first_and_last_name(client: Object, first_name: &String, last_name: &String) -> Vec<User> {    
    let res = client.query("SELECT id, first_name, last_name, birthdate, biography, city FROM users WHERE (first_name LIKE $1) AND (last_name LIKE $2) ORDER BY id", &[&format!("{}%", first_name), &format!("{}%", last_name)]).await.unwrap();
    res
        .into_iter()
        .map(|row| {
            let id: Option<uuid::Uuid> = row.get("id");
            User {
                id: id.map(|t| t.to_string()),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                birthdate: row.get("birthdate"),
                biography: row.get("biography"),
                city: row.get("city"),
            }
        })
        .collect()
}