use crate::services::auth_service::TokenClaims;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;
use tonic::{Code, Status};

pub async fn get_token_claims(token: &str, jwt_secret: &str) -> Result<TokenClaims, Status> {
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(jwt_secret.as_bytes()).expect("Cannot create hash from jwt secret");
    let claims = token
        .verify_with_key(&key)
        .map_err(|e| Status::new(Code::Unauthenticated, e.to_string()))?;

    Ok(claims)
}

use std::future::{ready, Ready};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;

pub struct JwtMiddleware {
    pub jwt_secret: String,
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareFactory<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let jwt_secret = self.jwt_secret.clone();
        ready(Ok(JwtMiddlewareFactory {
            service,
            jwt_secret,
        }))
    }
}

pub struct JwtMiddlewareFactory<S> {
    service: S,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareFactory<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Error = Error;
    type Response = ServiceResponse<EitherBody<B>>;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Some(header_value) = req.headers().get("authorization") {
            if let Ok(header_value) = header_value.to_str() {
                let token: String = header_value.chars().skip(7).collect();
                let key: Hmac<Sha256> = Hmac::new_from_slice(self.jwt_secret.as_bytes())
                    .expect("Cannot create hash from jwt secret");
                let claims: Result<TokenClaims, _> = token.verify_with_key(&key);
                if let Ok(claims) = claims {
                    HttpMessage::extensions_mut(&req).insert(claims);
                    let fut = self.service.call(req);
                    return Box::pin(
                        async move { fut.await.map(ServiceResponse::map_into_left_body) },
                    );
                }
            }
        }

        let (request, _) = req.into_parts();
        let response = HttpResponse::Unauthorized().finish().map_into_right_body();
        Box::pin(async { Ok(ServiceResponse::new(request, response)) })
    }
}
