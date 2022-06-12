use crate::chat_server::chat_server::{ChatServer, ChatType};
use crate::constants::DEFAULT_ROOM;
use crate::data_stores::elastic::store::ElasticStore;
use crate::jwt::lib::{JwtToken, Payload};
use crate::models::elastic::DocumentMetadata;
use crate::models::request_models::{LoginForm, RegistrationForm};
use crate::models::users::{RegisterUserResult, UserDocument};
use crate::session;
use actix::Addr;
use actix_web::{get, post, web, web::ReqData, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use serde_json::json;
use std::time::Instant;

#[post("/register")]
pub async fn register(
    elastic: web::Data<ElasticStore>,
    register_form: web::Json<RegistrationForm>,
) -> impl Responder {
    let username_invalid =
        register_form.username.trim() == "" || register_form.username.chars().count() < 4;
    let password_invalid =
        register_form.password.trim() == "" || register_form.password.chars().count() < 6;

    let email_invalid =
        register_form.email.trim() == "" || register_form.password.chars().count() < 6;

    if username_invalid || password_invalid || email_invalid {
        return HttpResponse::BadRequest().body("Bad Request");
    }

    match elastic.users.retrieve_user(&register_form.username).await {
        Ok(val) => {
            if val.hits.total.value > 0 {
                return HttpResponse::BadRequest().json(json!({"data": "User already exists"}));
            }
        }
        Err(error) => {
            if error.status_code().unwrap().as_str() != "404" {
                return HttpResponse::BadRequest().json(json!({"data": "Something went wrong"}));
            }
        }
    };

    let create_user_result: RegisterUserResult =
        match elastic.users.create_user(register_form).await {
            Ok(val) => val,
            Err(error) => {
                if error.status_code().unwrap().as_str() == "409" {
                    return HttpResponse::BadRequest().json(json!({"data": "User already exists"}));
                }
                return HttpResponse::BadRequest().json(json!({"data": "Something went wrong"}));
            }
        };
    HttpResponse::Created().json(create_user_result)
}

#[post("/login")]
pub async fn login(
    elastic: web::Data<ElasticStore>,
    login_form: web::Json<LoginForm>,
) -> impl Responder {
    let retrieve_user_result: DocumentMetadata<UserDocument> =
        match elastic.users.retrieve_user(&login_form.username).await {
            Ok(val) => val.hits.hits.get(0).unwrap().clone(),
            Err(error) => {
                if error.status_code().unwrap().as_str() == "404" {
                    return HttpResponse::BadRequest()
                        .json(json!({"data": "Either username or password was bad"}));
                }
                return HttpResponse::BadRequest().json(json!({"data": "Something went wrong"}));
            }
        };
    let valid = bcrypt::verify(&login_form.password, &retrieve_user_result._source.password);
    if !valid.unwrap() {
        return HttpResponse::BadRequest()
            .json(json!({"data": "Either username or password was bad"}));
    }

    let jwt_token = JwtToken::from_elastic_user_document(&retrieve_user_result, None);

    HttpResponse::Ok()
        .insert_header(("Authorization", jwt_token.token))
        .json(json!(retrieve_user_result))
}

#[get("/connect")]
pub async fn connect(
    req: HttpRequest,
    user: Option<ReqData<Payload>>,
    stream: web::Payload,
    srv: web::Data<Addr<ChatServer>>,
) -> impl Responder {
    let user_payload: Payload = user.unwrap().into_inner();

    ws::start(
        session::WsChatSession {
            username: user_payload.username.clone(),
            hb: Instant::now(),
            addr: srv,
            channel_name: DEFAULT_ROOM.to_owned(),
            valid_connection: false,
            chat_type: ChatType::Room,
        },
        &req,
        stream,
    )
    .unwrap()
}

#[get("/healthcheck")]
pub async fn healthcheck() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
