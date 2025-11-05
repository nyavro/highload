use deadpool_postgres::{Object};

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

pub async fn register_user<'a>(client: Object, req: UserRegistration<'a>) -> UserRegistrationResult {
    UserRegistrationResult {
        user_id: Some("1234".to_string())
    }
}