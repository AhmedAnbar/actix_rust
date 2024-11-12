use std::fmt::Display;

use actix_web::{body::BoxBody, http::StatusCode, web, HttpResponse, Responder, ResponseError};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponseError {
    pub message: String,
    pub status: u16,
    pub validation_errors: Option<Vec<ValidationErrorDetail>>,
}
impl ApiResponseError {
    pub fn new(
        status: u16,
        message: String,
        validation_errors: Option<Vec<ValidationErrorDetail>>,
    ) -> Self {
        Self {
            status,
            message,
            validation_errors,
        }
    }
}

impl Responder for ApiResponseError {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let json_body = serde_json::to_string(&self).unwrap();

        HttpResponse::build(StatusCode::from_u16(self.status).unwrap())
            .content_type("application/json")
            .body(BoxBody::new(web::BytesMut::from(json_body.as_bytes())))
    }
}

impl Display for ApiResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl ResponseError for ApiResponseError {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap()
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let json_body = serde_json::to_string(&self).unwrap();
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap())
            .content_type("application/json")
            .body(BoxBody::new(web::BytesMut::from(json_body.as_bytes())))
    }
}
