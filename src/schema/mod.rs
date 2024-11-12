use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

pub mod admin;
pub mod project;
pub mod response;

#[derive(Debug, Deserialize, IntoParams)]
pub struct FilterOptions {
    #[param(example = 1)]
    pub page: Option<usize>,
    #[param(example = 10)]
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct ParamOptions {
    pub id: String,
}
