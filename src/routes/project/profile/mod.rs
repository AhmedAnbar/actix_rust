use actix_web::web;

use crate::{
    handlers::project::profile::{
        get_profile::profile_handler, update_profile::update_profile_handler,
    },
    middlewares::auth_middleware::RequireAuth,
};

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/profile")
        .wrap(RequireAuth {})
        .service(update_profile_handler)
        .service(profile_handler);

    conf.service(scope);
}
