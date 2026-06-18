use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: uuid::Uuid, // user ID
    exp: usize,  // expiration time 
    pub token: Option<String>,
    pub request_id: Option<String>
}

pub fn create_token(user_id: &uuid::Uuid, secret: &[u8], token_ttl: i64) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        user_id: user_id.to_owned(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(token_ttl)).timestamp() as usize, 
        token: None, 
        request_id: None      
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
}

pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();    
    decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|token_data| {
            let mut claims = token_data.claims;
            claims.token = Some(token.to_string());            
            claims
        })        
}