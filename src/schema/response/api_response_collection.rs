use std::fmt::Display;

use actix_web::{body::BoxBody, http::StatusCode, web, HttpResponse, Responder, ResponseError};
use serde::Serialize;
use utoipa::ToSchema;

use super::{api_response_object::ApiResponseObject, Pagination};

#[derive(Serialize, ToSchema, Debug)]
pub struct ApiResponseCollection {
    pub status: u16,
    pub message: String,
    pub data: Option<ApiResponseObject>,
    pub pagination: Option<Pagination>,
}

impl ApiResponseCollection {
    pub fn new(
        status_code: u16,
        message: String,
        data: Option<ApiResponseObject>,
        pagination: Option<Pagination>,
    ) -> Self {
        Self {
            status: status_code,
            message,
            data,
            pagination,
        }
    }
}

impl Responder for ApiResponseCollection {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let json_body = serde_json::to_string(&self).unwrap();

        HttpResponse::build(StatusCode::from_u16(self.status).unwrap())
            .content_type("application/json")
            .body(BoxBody::new(web::BytesMut::from(json_body.as_bytes())))
    }
}

impl Display for ApiResponseCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl ResponseError for ApiResponseCollection {
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
