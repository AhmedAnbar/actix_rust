use crate::{
    handlers::health_checker::{health_checker_auth_handler, health_checker_handler},
    middlewares::auth_middleware::RequireAuth,
};
use actix_web::web;

pub fn config(conf: &mut web::ServiceConfig) {
    let auth_scope = web::scope("/check")
        .service(web::resource("").to(health_checker_handler))
        .service(
            web::resource("/auth")
                .to(health_checker_auth_handler)
                .wrap(RequireAuth {}),
        );

    conf.service(auth_scope);
}
