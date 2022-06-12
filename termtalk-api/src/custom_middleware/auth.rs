use std::future::{ready, Ready};

use crate::jwt::lib::JwtToken;
use actix_web::body::EitherBody;
use actix_web::dev::{self, ServiceRequest, ServiceResponse};
use actix_web::dev::{Service, Transform};
use actix_web::HttpMessage;
use actix_web::{Error, HttpResponse};
use futures_util::future::LocalBoxFuture;

pub struct Authenticate;

impl<S, B> Transform<S, ServiceRequest> for Authenticate
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}
pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        let exclude_paths = vec!["/healthcheck", "/register", "/login"];

        if exclude_paths.contains(&request.path()) {
            log::debug!(
                "The following path is excluded from the AuthenticationMiddleware: {}",
                request.path()
            );
        } else {
            let auth_header = request.headers().get("Authorization").unwrap();
            let auth_header_parts = auth_header
                .to_str()
                .unwrap()
                .split(" ")
                .collect::<Vec<&str>>();
            if auth_header_parts.len() != 2 {
                let (request, _pl) = request.into_parts();
                let response = HttpResponse::BadRequest()
                    .body("Bad Request")
                    .map_into_right_body();
                log::debug!("The Authorization header had too many parts or too few parts");
                return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
            }
            let token_payload = JwtToken::verify(auth_header_parts[1]);
            match token_payload {
                Ok(payload) => {
                    request.extensions_mut().insert(payload);
                }
                Err(e) => {
                    let (request, _pl) = request.into_parts();
                    let response = HttpResponse::BadRequest()
                        .body("Bad Request")
                        .map_into_right_body();
                    log::debug!("JwtToken is invalid: {:?}", e);
                    return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
                }
            };
        }

        let res = self.service.call(request);
        Box::pin(async move { res.await.map(ServiceResponse::map_into_left_body) })
    }
}
