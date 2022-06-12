use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RegistrationForm {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}
