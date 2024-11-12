use std::fmt::Display;

use actix_web::{body::BoxBody, http::StatusCode, web, HttpResponse, Responder, ResponseError};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::api_response_object::ApiResponseObject;

#[derive(Serialize, ToSchema, Debug, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub message: String,
    pub data: Option<ApiResponseObject>,
}

impl ApiResponse {
    pub fn new(status_code: u16, message: String, data: Option<ApiResponseObject>) -> Self {
        Self {
            status: status_code,
            message,
            data,
        }
    }
}

impl Responder for ApiResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let json_body = serde_json::to_string(&self).unwrap();

        HttpResponse::build(StatusCode::from_u16(self.status).unwrap())
            .content_type("application/json")
            .body(BoxBody::new(web::BytesMut::from(json_body.as_bytes())))
    }
}

impl Display for ApiResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl ResponseError for ApiResponse {
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
