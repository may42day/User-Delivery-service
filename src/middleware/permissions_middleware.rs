use crate::services::auth_service::TokenClaims;
use actix_web::dev::ServiceRequest;
use actix_web::dev::{forward_ready, Service, ServiceResponse, Transform};
use actix_web::error::ErrorForbidden;
use actix_web::{Error, HttpMessage};
use futures::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;

#[derive(Clone)]
pub struct PermissionsMiddleware<S> {
    permissions_policy: Rc<Vec<String>>,
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for PermissionsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let policy = self.permissions_policy.clone();
        let token_claims = req
            .extensions()
            .get::<TokenClaims>()
            .expect("Cannot parse TokenClaims from request-local data container")
            .clone();
        let srv = self.service.call(req);

        Box::pin(async move {
            let res = srv.await?;
            if policy.contains(&token_claims.role) {
                Ok(res)
            } else {
                Err(ErrorForbidden("Access denied"))
            }
        })
    }
}

#[derive(Clone)]
pub struct PermissionsMiddlewareFactory {
    permissions_policy: Rc<Vec<String>>,
}

impl PermissionsMiddlewareFactory {
    pub fn new(permissions: Vec<String>) -> Self {
        PermissionsMiddlewareFactory {
            permissions_policy: Rc::new(permissions),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for PermissionsMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = PermissionsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PermissionsMiddleware {
            permissions_policy: self.permissions_policy.clone(),
            service: Rc::new(service),
        }))
    }
}
