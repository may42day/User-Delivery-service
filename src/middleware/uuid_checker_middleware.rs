use crate::services::auth_service::TokenClaims;
use actix_web::dev::ServiceRequest;
use actix_web::dev::{forward_ready, Service, ServiceResponse, Transform};
use actix_web::error::ErrorForbidden;
use actix_web::{Error, HttpMessage};
use futures::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;

#[derive(Clone)]
pub struct UuidCheckerMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for UuidCheckerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token_claims = req
            .extensions()
            .get::<TokenClaims>()
            .expect("Cannot parse TokenClaims from request")
            .clone();
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            let uuid = res.request().match_info().query("uuid");
            if uuid == token_claims.uuid.to_string() || token_claims.role == *"ADMIN" {
                Ok(res)
            } else {
                Err(ErrorForbidden("Access denied"))
            }
        })
    }
}

#[derive(Clone, Default)]
pub struct UuidCheckerMiddlewareFactory {}

impl<S, B> Transform<S, ServiceRequest> for UuidCheckerMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = UuidCheckerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UuidCheckerMiddleware {
            service: Rc::new(service),
        }))
    }
}
