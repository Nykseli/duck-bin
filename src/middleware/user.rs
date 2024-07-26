use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;

use crate::data::{DataPool, User};

pub struct UserSession;

impl<S, B> Transform<S, ServiceRequest> for UserSession
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UserSessionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UserSessionMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct UserSessionMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for UserSessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let serv = self.service.clone();

        Box::pin(async move {
            let db = &req
                .request()
                .app_data::<Data<DataPool>>()
                .expect("Database pool doesn't exist")
                .pool;

            if let Some(user_secret) = req.request().cookie("user_secret") {
                let secret = user_secret.value();

                let user = sqlx::query_as!(
                    User,
                    "SELECT
                    users.* FROM users
                    LEFT JOIN user_sessions ON user_sessions.user_id=users.id
                    WHERE user_sessions.session_id=?",
                    secret
                )
                .fetch_one(db)
                .await;

                if let Ok(user) = user {
                    req.extensions_mut().insert(user);
                }
            }

            let fut = serv.call(req);
            let res = fut.await?;
            Ok(res)
        })
    }
}
