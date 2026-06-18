use serde::{Serialize, Deserialize};
use jsonwebtoken::{decode, DecodingKey, Validation};
use log::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: uuid::Uuid, // user ID
    exp: usize,  // expiration time
    pub token: Option<String>
}

pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();    
    info!("Verifying token {:?}", token);
    decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|token_data| token_data.claims)        
}