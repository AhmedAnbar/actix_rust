use actix_web::web;

use crate::{
    core::enums::UserRole,
    handlers::admin::user::{
        create_user::create_user_handler, delete_user::delete_user_handler,
        get_user_by_id::get_user_by_id_handler, get_users::get_users_handler,
        update_user::update_user_handler,
    },
    middlewares::auth_admin_middleware::RequireAdminAuth,
};

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/users")
        .wrap(RequireAdminAuth::allowed_roles(vec![UserRole::Admin]))
        .service(get_users_handler)
        .service(get_user_by_id_handler)
        .service(create_user_handler)
        .service(delete_user_handler)
        .service(update_user_handler);

    conf.service(scope);
}
