use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, PasswordHash, PasswordVerifier, SaltString},Argon2};

pub fn hash_password(password: String) -> (SaltString, String) {
    let salt = SaltString::generate(&mut OsRng); 
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();
    (salt, password_hash)
}

pub fn check_password(password: &String, hash: String) -> bool {
    let parsed_hash = PasswordHash::new(&hash).unwrap();    
    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => true, 
        Err(_) => false
    }
}