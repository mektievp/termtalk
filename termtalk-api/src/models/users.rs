use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDocument {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterUserResult {
    _id: String,
    _index: String,
    result: String,
}

#[derive(Debug)]
pub struct CreateUserError;
impl Error for CreateUserError {}

impl fmt::Display for CreateUserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}
