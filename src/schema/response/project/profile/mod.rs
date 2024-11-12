use crate::model::user::UserModel;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[allow(non_snake_case)]
pub struct ProfileResponse {
    pub id: String,
    pub name: String,
    pub mobile: String,
    pub email: String,
    pub gender: String,
    pub roleId: i32,
    pub active: bool,
    pub protected: bool,
}

impl ProfileResponse {
    pub fn new(profile: &UserModel) -> Self {
        Self {
            id: profile.id.to_owned(),
            name: profile.name.to_owned(),
            mobile: profile.mobile.to_owned(),
            email: profile.email.to_owned().unwrap(),
            gender: profile.gender.to_owned().unwrap(),
            roleId: profile.role_id.to_owned(),
            active: profile.active != 0,
            protected: profile.protected != 0,
        }
    }
}
