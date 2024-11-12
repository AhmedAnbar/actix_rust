pub mod contents_table;
pub mod users_table;

use actix_web::web;
use contents_table::create_fake_contents;
use users_table::create_fake_users;

pub fn config(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/fake")
            .service(create_fake_users)
            .service(create_fake_contents),
    );
}
