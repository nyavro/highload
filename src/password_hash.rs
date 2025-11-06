use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString},Argon2};

pub fn hash_password(password: String) -> (SaltString, String) {
    let salt = SaltString::generate(&mut OsRng); 
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();
    (salt, password_hash)
}