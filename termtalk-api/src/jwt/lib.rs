use crate::models::elastic::DocumentMetadata;
use crate::models::users::{User, UserDocument};
use base64_url;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::env;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

static ONE_DAY_IN_SECONDS: u64 = 86400;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct JwtToken {
    header: String,
    payload: String,
    signature: String,
    iat: u64,
    pub token: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Header {
    alg: String,
    typ: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Payload {
    pub id: String,
    pub username: String,
    pub email: String,
    pub iat: u64,
}

impl JwtToken {
    pub fn new(iat: Option<u64>) -> JwtToken {
        let issued_at: u64 = match iat {
            Some(val) => val,
            None => time_as_secs_since_epoch(),
        };
        JwtToken {
            header: Default::default(),
            payload: Default::default(),
            token: Default::default(),
            signature: Default::default(),
            iat: issued_at,
        }
    }

    pub fn generate_jwt_token_from_user(&mut self, user: User) {
        self.generate_and_set_jwt_header();
        self.generate_and_set_jwt_payload(user);
        self.generate_and_set_jwt_signature();
        self.string_format_jwt_token();
    }

    fn generate_and_set_jwt_header(&mut self) {
        let header = Header {
            alg: String::from("HS256"),
            typ: String::from("JWT"),
        };
        let header_string = serde_json::to_string(&header).unwrap();
        let header_base64_encoded = base64_url::encode(&header_string);

        self.header = header_base64_encoded.trim_end_matches("=").to_string();
    }

    pub fn generate_and_set_jwt_payload(&mut self, user: User) {
        let payload = Payload {
            id: user.id,
            username: user.username,
            email: user.email,
            iat: self.iat,
        };
        let payload_string = serde_json::to_string(&payload).unwrap();
        let payload_base64_encoded = base64_url::encode(&payload_string);
        self.payload = payload_base64_encoded.trim_end_matches("=").to_string();
    }

    pub fn generate_and_set_jwt_signature(&mut self) {
        let hmacsha256_input: String = self.format_header_and_payload_for_hmacsha256_input();
        let secret_key = env::var("SECRET_KEY").unwrap();
        let sha256_algorithm = Sha256::new();

        let mut new_hmac = Hmac::new(sha256_algorithm, secret_key.as_bytes());
        new_hmac.input(hmacsha256_input.as_bytes());
        let hmac = new_hmac.result();

        let result = hmac.code();
        let base64_encoded: String = base64_url::encode(&result);
        self.signature = base64_encoded.trim_end_matches("=").to_string();
    }

    fn string_format_jwt_token(&mut self) {
        let mut jwt_token: String = String::from("");
        jwt_token.push_str(&self.header);
        jwt_token.push_str(".");
        jwt_token.push_str(&self.payload);
        jwt_token.push_str(".");
        jwt_token.push_str(&self.signature);
        self.token = jwt_token;
    }

    fn format_header_and_payload_for_hmacsha256_input(&self) -> String {
        let mut hmacsha256_input: String = self.header.to_string();
        hmacsha256_input.push_str(".");
        hmacsha256_input.push_str(&self.payload);

        return hmacsha256_input;
    }

    pub fn create_from_user(user: User, issued_at: Option<u64>) -> JwtToken {
        let mut jwt_token: JwtToken = JwtToken::new(issued_at);
        jwt_token.generate_jwt_token_from_user(user);
        return jwt_token;
    }

    pub fn from_elastic_user_document(
        elastic_user_doc: &DocumentMetadata<UserDocument>,
        issued_at: Option<u64>,
    ) -> JwtToken {
        let user = User {
            id: elastic_user_doc._id.clone(),
            username: elastic_user_doc._source.username.clone(),
            email: elastic_user_doc._source.email.clone(),
            password: String::from(""),
        };
        let mut jwt_token: JwtToken = JwtToken::new(issued_at);
        jwt_token.generate_jwt_token_from_user(user);
        return jwt_token;
    }

    pub fn verify(unverified_token: &str) -> Result<Payload, InvalidJwtToken> {
        let jwt_token_parts = unverified_token.split(".").collect::<Vec<&str>>();
        if jwt_token_parts.len() != 3 {
            return Err(InvalidJwtToken::BadFormat);
        }
        let unverified_token_payload = jwt_token_parts[1];

        let payload_decoded = base64_url::decode(unverified_token_payload).unwrap();

        let payload_string = str::from_utf8(&payload_decoded).unwrap();

        let payload: Payload = serde_json::from_str(payload_string).unwrap();
        let payload_clone = payload.clone();
        let unverified_user = User {
            id: payload.id,
            username: payload.username,
            email: payload.email,
            password: String::from(""),
        };

        let verified_jwt_token = JwtToken::create_from_user(unverified_user, Some(payload.iat));

        if verified_jwt_token.token != unverified_token {
            return Err(InvalidJwtToken::Unauthorized);
        }

        let twenty_four_hours_ago = time_as_secs_since_epoch() - ONE_DAY_IN_SECONDS;
        if twenty_four_hours_ago > payload.iat {
            return Err(InvalidJwtToken::Expired);
        }

        return Ok(payload_clone);
    }
}

#[derive(Debug, PartialEq)]
pub enum InvalidJwtToken {
    BadFormat,
    Unauthorized,
    Expired,
}

fn time_as_secs_since_epoch() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards!");

    return since_the_epoch.as_secs();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_generate_and_set_jwt_header() {
        let mut jwt_token: JwtToken = JwtToken::new(None);
        jwt_token.generate_and_set_jwt_header();

        assert_eq!(expected_jwt_header(), jwt_token.header);
    }

    #[test]
    fn test_generate_and_set_jwt_payload() {
        let user = user_factory();
        let mut jwt_token = jwt_token_factory();

        jwt_token.generate_and_set_jwt_payload(user);
        assert_eq!(expected_jwt_payload(), jwt_token.payload);
    }

    #[test]
    fn test_format_header_and_payload_for_hmacsha256_input() {
        let mut jwt_token = jwt_token_factory();
        let user = user_factory();
        jwt_token.generate_and_set_jwt_header();
        jwt_token.generate_and_set_jwt_payload(user);

        let hmacsha256_input = jwt_token.format_header_and_payload_for_hmacsha256_input();
        assert_eq!(
            format!("{}.{}", expected_jwt_header(), expected_jwt_payload()),
            hmacsha256_input
        );
    }

    #[test]
    fn test_generate_and_set_jwt_signature() {
        let mut jwt_token = jwt_token_factory();
        let user = user_factory();

        jwt_token.generate_and_set_jwt_header();
        jwt_token.generate_and_set_jwt_payload(user);
        jwt_token.generate_and_set_jwt_signature();

        assert_eq!(expected_jwt_signature(), jwt_token.signature);
    }

    #[test]
    fn test_string_format_jwt_token() {
        let mut jwt_token = jwt_token_factory();
        let user = user_factory();
        jwt_token.generate_and_set_jwt_header();
        jwt_token.generate_and_set_jwt_payload(user);
        jwt_token.generate_and_set_jwt_signature();
        jwt_token.string_format_jwt_token();
        assert_eq!(expected_jwt_token(), jwt_token.token);
    }

    #[test]
    fn test_generate_jwt_token_from_user() {
        let mut jwt_token = jwt_token_factory();
        let user = user_factory();
        jwt_token.generate_jwt_token_from_user(user);

        assert_eq!(expected_jwt_header(), jwt_token.header);
        assert_eq!(expected_jwt_payload(), jwt_token.payload);
        assert_eq!(expected_jwt_signature(), jwt_token.signature);
        assert_eq!(expected_jwt_token(), jwt_token.token);
        assert_eq!(jwt_iat_factory(), jwt_token.iat);
    }

    #[test]
    fn test_create_from_user() {
        let user = user_factory();
        let jwt_token = JwtToken::create_from_user(user, Some(jwt_iat_factory()));
        assert_eq!(expected_jwt_header(), jwt_token.header);
        assert_eq!(expected_jwt_payload(), jwt_token.payload);
        assert_eq!(expected_jwt_signature(), jwt_token.signature);
        assert_eq!(expected_jwt_token(), jwt_token.token);
        assert_eq!(jwt_iat_factory(), jwt_token.iat);
    }

    #[test]
    fn test_verify_jwt_token_with_valid_token() {
        std::env::set_var("SECRET_KEY", "SECRET_KEY");
        let token_payload = JwtToken::verify(&expected_jwt_token()).unwrap();
        let expected_payload = Payload {
            id: String::from("1"),
            username: String::from("zalir"),
            email: String::from("mektievp@gmail.com"),
            iat: jwt_iat_factory(),
        };
        assert_eq!(expected_payload, token_payload);
    }

    #[test]
    fn test_verify_jwt_token_with_invalid_token() {
        std::env::set_var("SECRET_KEY", "SECRET_KEY");
        let invalid_jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6IjEiLCJ1c2VybmFtZSI6InphbGlyIiwiZW1haWwiOiJtZWt0aWV2cEBnbWFpbC5jb20iLCJpYXQiOjE2NDk0ODc4OTMzNzF9.bBjusV1s4ZTFylmbzX3yyuB6pacuu0lg5iA1_J4HT4E";
        let token_payload = JwtToken::verify(invalid_jwt_token);
        let jwt_invalid_token_error = match token_payload {
        Err(e) => e,
        Ok(_) => panic!("test_verify_jwt_token_with_invalid_token was supposed to panic with InvalidJwtToken::Unauthorized error")
      };
        assert_eq!(InvalidJwtToken::Unauthorized, jwt_invalid_token_error);
    }

    #[test]
    fn test_jwt_token_bad_format() {
        std::env::set_var("SECRET_KEY", "SECRET_KEY");
        let jwt_bad_format = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6IjEiLCJ1c2VybmFtZSI6InphbGlyIiwiZW1haWwiOiJtZWt0aWV2cEBnbWFpbC5jb20iLCJpYXQiOjE2NDk0ODc4OTMzNzF9.bBjusV1s4ZTFylmbzX3yyuB6pacuu0lg5iA1_J4HT4E";
        let token_payload = JwtToken::verify(jwt_bad_format);
        let jwt_bad_format_error = match token_payload {
        Err(e) => e,
        Ok(_) => panic!("test_jwt_token_bad_format was supposed to panic with InvalidJwtToken::BadFormat error")
      };
        assert_eq!(InvalidJwtToken::BadFormat, jwt_bad_format_error);
    }

    fn user_factory() -> User {
        User {
            id: String::from("1"),
            username: String::from("zalir"),
            email: String::from("mektievp@gmail.com"),
            password: String::from("password"),
        }
    }

    fn jwt_iat_factory() -> u64 {
        1516239022
    }

    fn jwt_token_factory() -> JwtToken {
        std::env::set_var("SECRET_KEY", "SECRET_KEY");
        let issued_at: Option<u64> = Some(jwt_iat_factory());
        JwtToken::new(issued_at)
    }

    fn expected_jwt_header() -> &'static str {
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"
    }

    fn expected_jwt_payload() -> &'static str {
        "eyJpZCI6IjEiLCJ1c2VybmFtZSI6InphbGlyIiwiZW1haWwiOiJtZWt0aWV2cEBnbWFpbC5jb20iLCJpYXQiOjE2NDk0ODc4OTMzNzF9"
    }

    fn expected_jwt_signature() -> &'static str {
        "EFDZpOmHXFaJVIG9IWJRcrXtYi5lF0v9bftxboNoAxU"
    }

    fn expected_jwt_token() -> String {
        format!(
            "{}.{}.{}",
            expected_jwt_header(),
            expected_jwt_payload(),
            expected_jwt_signature()
        )
        .to_string()
    }
}
