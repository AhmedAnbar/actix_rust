use actix_web::web;

use crate::{
    handlers::auth::{
        login::login_user_handler, logout::logout_user_handler, register::register_user_handler,
        verify::verify_otp_handler,
    },
    middlewares::auth_middleware::RequireAuth,
};

// pub fn config(conf: &mut web::ServiceConfig) {
//     let scope = web::scope("/auth")
//         .service(register_user_handler)
//         .service(login_user_handler)
//         .service(logout_user_handler)
//         .service(verify_otp_handler);
//
//     conf.service(scope);
// }

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/auth")
        .service(register_user_handler)
        .service(login_user_handler)
        .service(
            web::resource("/logout")
                .wrap(RequireAuth {})
                .route(web::post().to(logout_user_handler)),
        )
        .service(verify_otp_handler);

    conf.service(scope);
}
