use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::{
    handlers::{ 
        admin::{
            contents::{create_contents::__path_create_contents_handler, delete_content::__path_delete_contents_handler, get_content_by_id::__path_get_content_by_id_handler, get_contents::__path_get_contents_handler, update_contents::__path_update_contents_handler}, 
            user::{create_user::__path_create_user_handler, delete_user::__path_delete_user_handler, get_user_by_id::__path_get_user_by_id_handler, get_users::__path_get_users_handler, update_user::__path_update_user_handler}
        },
        auth::{
            login::{LoginUserRequest, __path_login_user_handler},
            register::{RegisterUserRequest, __path_register_user_handler},
            verify::{VerifyOtpRequest, __path_verify_otp_handler},
            logout::__path_logout_user_handler,
        },
        health_checker::{__path_health_checker_auth_handler, __path_health_checker_handler},
        project::profile::{
            get_profile::__path_profile_handler, update_profile::__path_update_profile_handler,
        },
    },
    schema::{admin::{content::{ContentsFilterOptions, CreateContentSchema, UpdateContentSchema}, user::{CreateUserSchema, UpdateUserSchema, UsersFilterOptions}}, project::profile::update_profile::UpdateProfileSchema, response::{api_response::ApiResponse, api_response_collection::ApiResponseCollection, api_response_error::{ApiResponseError, ValidationErrorDetail}, api_response_object::ApiResponseObject, Pagination}},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Health Checker
        health_checker_handler,
        health_checker_auth_handler,
        // Profile
        profile_handler,
        update_profile_handler,
        //Auth
        login_user_handler,
        logout_user_handler,
        verify_otp_handler,
        register_user_handler,
        // Admin-Users
        get_users_handler,
        get_user_by_id_handler,
        create_user_handler,
        update_user_handler,
        delete_user_handler,
        // Admin Contents
        get_contents_handler,
        get_content_by_id_handler,
        create_contents_handler,
        update_contents_handler,
        delete_contents_handler,
    ),
    components(
        schemas(
            ApiResponse, ApiResponseCollection, ApiResponseObject, ApiResponseError, Pagination, ValidationErrorDetail,
            UpdateProfileSchema,
            CreateContentSchema, UpdateContentSchema, ContentsFilterOptions,
            LoginUserRequest, VerifyOtpRequest, RegisterUserRequest,
            CreateUserSchema, UpdateUserSchema, UsersFilterOptions
        )
    ),
    tags(
        (name = "Health Checker Endpoint", description = "Health Checker Endpoint"),
        (name = "Auth Endpoint", description = "Authenticated endpoints: Login, VerifyOTP, Register"),
        (name = "Profile Endpoint", description = "Get Profile and Update Profile"),
        (name = "Admin: Users Endpoint", description = "Admin User management: Create User, Get Users, Update User, Delete User, Get User By ID"),
        (name = "Admin: Contents Endpoint", description = "Admin Content management: Create Contetns, Get Contents, Update Contents, Delete Contents, Get Content By ID"),
        
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "auth_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}
