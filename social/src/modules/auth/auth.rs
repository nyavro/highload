use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: uuid::Uuid, // user ID
    exp: usize,  // expiration time
}

pub fn create_token(user_id: &uuid::Uuid, secret: &[u8], token_ttl: i64) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        user_id: user_id.to_owned(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(token_ttl)).timestamp() as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
}

pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();    
    decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|token_data| token_data.claims)        
}