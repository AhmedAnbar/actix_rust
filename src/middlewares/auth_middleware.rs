use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{http, web, HttpMessage};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use futures_util::FutureExt;
use jsonwebtoken::errors::ErrorKind;
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::core::app_state::AppState;
use crate::core::utils::jwt::decode_jwt;
use crate::model::user::UserModel;
use crate::schema::response::admin::users::AuthUser;
use crate::schema::response::api_response_error::ApiResponseError;

pub struct RequireAuth {}

impl<S> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

/// Middleware responsible for handling authentication and user information extraction.
pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

    /// Polls the readiness of the wrapped service.
    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    /// Handles incoming requests.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Attempt to extract token from cookie or authorization header
        let token = req
            .cookie("auth_token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        let dd = req.cookie("auth_token").map(|c| c.value().to_string());
        match dd {
            Some(cookie_value) => {
                println!("Cookie value: {}", cookie_value);
                // ApiResponseError::new(400, "fail".to_string(), None)
                // let json_error = ApiResponseError::new(400, "fail".to_string(), None);
                // return Box::pin(ready(Err(ErrorUnauthorized(json_error))));
            }
            None => {
                println!("Cookie not found");
                // let json_error = ApiResponseError::new(400, "fail".to_string(), None);
                // return Box::pin(ready(Err(ErrorUnauthorized(json_error))));
            }
        }
        // If token is missing, return unauthorized error
        if token.is_none() {
            let json_error = ApiResponseError::new(400, "fail".to_string(), None);
            return Box::pin(ready(Err(ErrorUnauthorized(json_error))));
        }

        let app_state = req.app_data::<web::Data<AppState>>().unwrap().clone();
        let srv = Rc::clone(&self.service);
        let token = token.unwrap();

        let token = token.replace("Bearer ", "");
        // Decode token and handle errors
        let claim = match decode_jwt(token) {
            Ok(claim) => claim,
            Err(err) => {
                if err == ErrorKind::InvalidToken.into()
                    || err == ErrorKind::InvalidSignature.into()
                {
                    return Box::pin(ready(Err(ErrorUnauthorized(ApiResponseError::new(
                        401,
                        "Invlaid token".to_string(),
                        None,
                    )))));
                } else if err == ErrorKind::ExpiredSignature.into() {
                    return Box::pin(ready(Err(ErrorUnauthorized(ApiResponseError::new(
                        401,
                        "Token is expired".to_string(),
                        None,
                    )))));
                }
                return Box::pin(ready(Err(ErrorUnauthorized(ApiResponseError::new(
                    401,
                    "Unauthorized-1".to_string(),
                    None,
                )))));
            }
        };

        // Handle user extraction and request processing
        async move {
            // Query user from database based on decoded user ID
            let query_result = sqlx::query_as!(
                UserModel,
                "SELECT * FROM users WHERE id = ?",
                claim.claims.id
            )
            .fetch_one(&app_state.pool)
            .await;

            // Handle query result
            let auth_data = match query_result {
                Ok(mut user) => AuthUser::filter_db(&mut user),
                Err(e) => {
                    return Err(ErrorInternalServerError(ApiResponseError::new(
                        500,
                        e.to_string(),
                        None,
                    )))
                }
            };

            // Insert user information into request extensions
            req.extensions_mut().insert::<AuthUser>(auth_data);

            // Call the wrapped service to handle the request
            let res = srv.call(req).await.map_err(|e| {
                ErrorInternalServerError(ApiResponseError::new(500, e.to_string(), None))
            })?;
            Ok(res)
        }
        .boxed_local()
    }
}
