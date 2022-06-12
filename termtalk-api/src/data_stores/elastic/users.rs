use crate::models::elastic::TermQuery;
use crate::models::request_models::RegistrationForm;
use crate::models::users::{RegisterUserResult, UserDocument};
use actix_web::web;
use elasticsearch;
extern crate bcrypt;
use bcrypt::{hash, DEFAULT_COST};
use serde_json::json;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UsersElasticStore {
    elastic: elasticsearch::Elasticsearch,
}

static USERS: &str = "users";

impl UsersElasticStore {
    pub fn new(elastic: elasticsearch::Elasticsearch) -> UsersElasticStore {
        Self {
            elastic: elastic.clone(),
        }
    }

    pub async fn retrieve_user(
        &self,
        username: &str,
    ) -> Result<TermQuery<UserDocument>, elasticsearch::Error> {
        println!("username: {}", username);
        let resp_body = self
            .elastic
            .search(elasticsearch::SearchParts::Index(&vec![USERS]))
            .body(json!({
                "query": {
                    "term": {
                        "username": {
                            "value": username
                        }
                    }
                }
            }))
            .send()
            .await
            .unwrap();

        let resp_result: elasticsearch::http::response::Response =
            match resp_body.error_for_status_code() {
                Ok(val) => val,
                Err(error) => return Err(error),
            };
        resp_result.json::<TermQuery<UserDocument>>().await
    }

    pub async fn create_user(
        &self,
        register_form: web::Json<RegistrationForm>,
    ) -> Result<RegisterUserResult, elasticsearch::Error> {
        let hashed: String = hash(&register_form.password, DEFAULT_COST).unwrap();
        let user_guid = Uuid::new_v4();
        let resp_body = self
            .elastic
            .index(elasticsearch::IndexParts::IndexId(
                USERS,
                &user_guid.to_string(),
            ))
            .body(json!({
                "username": &register_form.username,
                "email": &register_form.email,
                "password": hashed,
            }))
            .send()
            .await
            .unwrap();

        let resp_result: elasticsearch::http::response::Response =
            match resp_body.error_for_status_code() {
                Ok(val) => val,
                Err(error) => return Err(error),
            };

        resp_result.json::<RegisterUserResult>().await
    }
}
