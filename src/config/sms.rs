use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sms {
    pub enable: bool,
    pub account: String,
    pub token: String,
    pub from: String,
}
