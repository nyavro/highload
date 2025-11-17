use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: uuid::Uuid, // user ID
    exp: usize,  // expiration time
}

pub fn create_token(user_id: &uuid::Uuid, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(1)).timestamp() as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
}

fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256); 
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)?;
    Ok(token_data.claims)
}