use serde::{Serialize, Deserialize};
use jsonwebtoken::{decode, DecodingKey, Validation};


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: uuid::Uuid, // user ID
    exp: usize,  // expiration time
}

pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();    
    decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|token_data| token_data.claims)        
}