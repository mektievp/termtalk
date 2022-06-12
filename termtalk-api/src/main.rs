mod chat_server;
mod constants;
mod custom_middleware;
mod data_stores;
mod jwt;
mod models;
mod routes;
mod session;

use actix::Actor;
use actix_web::{middleware, web, App, HttpServer};
use constants::{REDIS_HOST, REDIS_PORT, TERMTALK_API_HOST, TERMTALK_API_PORT};
use data_stores::{
    elastic::store::ElasticStore,
    redis::{publish_chat_messages::CHAT_MESSAGES, store::RedisStore},
};
use routes::{connect, healthcheck, login, register};
use std::env;
use std::sync::Arc;
use std::thread;

use chat_server::{
    chat_server::{ChatServer, QueueMessage},
    handlers::send_client_message::SendClientMessage,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let redis_client: redis::Client = redis::Client::open(format!(
        "redis://{}:{}",
        env::var(REDIS_HOST).unwrap(),
        env::var(REDIS_PORT).unwrap()
    ))
    .unwrap();
    let elastic_client = elasticsearch::Elasticsearch::default();

    let redis_store: RedisStore = RedisStore::new(redis_client.clone());
    let elastic_store: ElasticStore = ElasticStore::new(elastic_client.clone());

    let chat_server: actix::Addr<ChatServer> =
        ChatServer::new(redis_store.clone(), elastic_store.clone())
            .await
            .start();

    let pubsub_chat_server_ = Arc::new(chat_server.clone());
    let pubsub_redis_store = Arc::new(redis_store.clone());

    thread::spawn(move || {
        let chat_server = Arc::clone(&pubsub_chat_server_);
        let redis_store = Arc::clone(&pubsub_redis_store);
        let mut redis_conn = redis_store.redis.get_connection().unwrap();
        let mut pubsub = redis_conn.as_pubsub();
        pubsub.subscribe(CHAT_MESSAGES).unwrap();

        loop {
            let msg: redis::Msg = pubsub.get_message().unwrap();
            let msg_payload: String = msg.get_payload().unwrap();
            let msg_deserialized: QueueMessage = serde_json::from_str(&msg_payload).unwrap();
            chat_server.do_send(SendClientMessage {
                sender: msg_deserialized.sender.clone(),
                msg: msg_deserialized.msg.clone(),
                chat_type: msg_deserialized.chat_type,
                msg_type: msg_deserialized.msg_type,
                recipient: msg_deserialized.recipient.clone(),
            });
        }
    });

    let termtalk_api_host = env::var(TERMTALK_API_HOST).unwrap();
    let termtalk_api_port = env::var(TERMTALK_API_PORT).unwrap().parse::<u16>().unwrap();
    log::info!(
        "Starting server on {}:{}",
        termtalk_api_host,
        termtalk_api_port
    );
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis_store.clone()))
            .app_data(web::Data::new(elastic_store.clone()))
            .app_data(web::Data::new(chat_server.clone()))
            .wrap(custom_middleware::auth::Authenticate)
            .wrap(middleware::Logger::default())
            .service(healthcheck)
            .service(register)
            .service(login)
            .service(connect)
    })
    .bind((termtalk_api_host, termtalk_api_port))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;
    use actix_web::dev::Service;
    use actix_web::{http, test, App, Error};

    #[actix_web::test]
    async fn test_healthcheck() -> Result<(), Error> {
        let app = App::new().service(healthcheck);
        let app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/healthcheck").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = resp.into_body();
        assert_eq!(to_bytes(response_body).await.unwrap(), r##"OK"##);

        Ok(())
    }
}
