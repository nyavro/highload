use deadpool_postgres::{Object};
use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString},Argon2};

#[derive(Debug)]
pub struct UserRegistration<'a> {
    pub first_name: &'a Option<String>,
    pub second_name: &'a Option<String>,
    pub birthdate: &'a Option<chrono::naive::NaiveDate>,
    pub biography: &'a Option<String>,
    pub city: &'a Option<String>,
    pub password: &'a Option<String>,
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
    let second_name = req.second_name.clone().expect("No second name specified");    
    match check_if_user_exists(&client, &second_name).await {
        Ok(_) => {
            let password = req.password.clone().expect("No password specified");                    
            let salt = SaltString::generate(&mut OsRng); 
            let password_hash = Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .expect("Failed to hash password")
                .to_string();
            
            let statement = client.prepare_cached("INSERT INTO users (first_name, second_name, biography, city, pwd, salt) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id").await.unwrap();
            let res = client.query_one(&statement, &[
                &req.first_name.clone().expect("No first name specified"),
                &req.second_name.clone().expect("No second name specified"),
                //&req.birthdate.map(|t| t),
                &req.biography,
                &req.city,
                &password_hash,
                &salt.to_string()
            ]).await.unwrap();
            let id: i32 = res.get(0);
            Ok(UserRegistrationResult {
                user_id: Some(id.to_string())
            })
        },
        Err(e) => Err(e)
    }    
}