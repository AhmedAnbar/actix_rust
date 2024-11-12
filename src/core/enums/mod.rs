use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Moderator,
    User,
}

impl UserRole {
    pub fn to_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::Moderator => "moderator",
            UserRole::User => "user",
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(UserRole::Admin),
            2 => Some(UserRole::Moderator),
            3 => Some(UserRole::User),
            _ => None,
        }
    }
}
