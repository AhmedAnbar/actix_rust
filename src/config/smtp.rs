use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Smtp {
    pub server: String,
    pub username: String,
    pub password: String,
    pub port: String,
    pub encryption: String,
    pub from: String,
}
