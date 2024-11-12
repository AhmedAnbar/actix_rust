use actix_web::web;

use crate::{
    core::enums::UserRole,
    handlers::admin::contents::{
        create_contents::create_contents_handler, delete_content::delete_contents_handler,
        get_content_by_id::get_content_by_id_handler, get_contents::get_contents_handler,
        update_contents::update_contents_handler,
    },
    middlewares::auth_admin_middleware::RequireAdminAuth,
};

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/contents")
        .wrap(RequireAdminAuth::allowed_roles(vec![UserRole::Admin]))
        .service(get_contents_handler)
        .service(get_content_by_id_handler)
        .service(create_contents_handler)
        .service(delete_contents_handler)
        .service(update_contents_handler);

    conf.service(scope);
}
