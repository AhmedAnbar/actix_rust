use actix_web::{put, web};
use uuid::Uuid;

use crate::{
    core::app_state::AppState,
    model::user::UserModel,
    schema::{
        admin::user::UpdateUserSchema,
        response::{
            admin::users::UserModelResponse, api_response::ApiResponse,
            api_response_error::ApiResponseError, api_response_object::ApiResponseObject,
        },
    },
};

#[utoipa::path(
    put,
    path = "/admin/users/update/{id}",
    tag = "Admin: Users Endpoint",
    params(
        ("id" = Uuid, Path, description = "UUID of the user to update"),
    ),
    request_body(content = UpdateUserSchema, description = "Credentials to update user", example = json!({"name": "Ahmed","mobile": "0123456789","email": "ahmed@example.com","gender": "Male", "active": true, "protected": false })),
    responses(
        (status = 204, description= "User Updated", body = ApiResponse),       
        (status = 401, description= "Unauthorized", body = ApiResponseError),       
        (status = 403, description= "User Is Protected", body = ApiResponseError),       
        (status = 404, description= "User Not Found", body = ApiResponseError),       
        (status = 500, description= "Internal Server Error", body = ApiResponseError),       
    ),
    security(
       ("auth_token" = [])
   )
)]
#[put("/update/{id}")]
pub async fn update_user_handler(
    path: web::Path<Uuid>,
    app_state: web::Data<AppState>,
    body: web::Json<UpdateUserSchema>,
) -> Result<ApiResponse, ApiResponseError> {
    let user_id = path.into_inner().to_string();

    let user = match sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_one(&app_state.pool)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            return Err(ApiResponseError::new(
                404,
                format!("User not found: {:?}", e),
                None,
            ));
        }
    };

    if user.protected == 1 {
        return Err(ApiResponseError::new(
            403,
            "User is protected".to_string(),
            None,
        ));
    }

    let active = body.active.unwrap_or(user.active != 0);
    let i8_active = active as i8;
    let protected = body.protected.unwrap_or(user.protected != 0);
    let i8_protected = protected as i8;

    let update_result = sqlx::query("UPDATE users SET name = ?, mobile = ?, email = ?, gender = ?, role_id = ?, active = ?, protected = ? WHERE id = ?")
        .bind(body.name.to_owned().unwrap_or_else(|| user.name.clone())) // Binds name
        .bind(body.mobile.to_owned().unwrap_or_else(|| user.mobile.clone())) // Binds mobile
        .bind(body.email.to_owned().unwrap_or_else(|| user.email.clone().unwrap_or_default())) // Binds email
        .bind(body.gender.to_owned().unwrap_or_else(|| user.gender.clone().unwrap_or_default())) // Binds gender
        .bind(body.role_id.to_owned().unwrap_or_else(|| user.role_id.clone())) // Binds role_id
        .bind(i8_active)
        .bind(i8_protected)
        .bind(user_id.clone())
        .execute(&app_state.pool)
        .await;

    match update_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                return Err(ApiResponseError::new(
                    500,
                    format!("User not updated"),
                    None,
                ));
            }
        }
        Err(e) => {
            return Err(ApiResponseError::new(
                500,
                format!("Internal server error: {:?}", e),
                None,
            ));
        }
    }

    let updated_user = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_one(&app_state.pool)
        .await;

    match updated_user {
        Ok(mut user) => {
            let user_response = ApiResponseObject::new(serde_json::json!({
                "user": UserModelResponse::filter_db(&mut user)
            }))
            .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?;
            return Ok(ApiResponse::new(
                200,
                "User updated".to_string(),
                Some(user_response),
            ));
        }
        Err(e) => {
            return Err(ApiResponseError::new(
                500,
                format!("Internal server error: {:?}", e),
                None,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::utils::test_utils::{create_test_app_state, generate_test_jwt},
        middlewares::auth_middleware::RequireAuth,
        model::user::UserModel,
        routes,
        schema::{
            admin::user::{CreateUserSchema, UpdateUserSchema},
            response::api_response::ApiResponse,
        },
    };
    use actix_web::{test, web, App};
    use fake::{
        faker::{internet::en::SafeEmail, name::en::Name},
        Fake,
    };
    use rand::Rng;

    #[actix_web::test]
    async fn test_update_user_handler() {
        let app_state = create_test_app_state().await;

        // create and configure the test app
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .wrap(RequireAuth {})
                .service(web::scope("/admin").configure(routes::admin::user::config)),
        )
        .await;

        let user_id = uuid::Uuid::new_v4().to_string();
        let jwt = generate_test_jwt();

        // insert test user data into the database
        let mobile = format!("9665{}", rand::thread_rng().gen_range(10000000..99999999));
        let create_user_data = CreateUserSchema {
            name: Some(Name().fake()),
            mobile: mobile.clone(),
            email: Some(SafeEmail().fake()),
        };

        let _insert_result = sqlx::query(
            "insert into users (id, name, mobile, email, protected) values (?, ?, ?, ?, ?, ?)",
        )
        .bind(&user_id)
        .bind(&create_user_data.name.as_deref())
        .bind(&create_user_data.mobile)
        .bind(create_user_data.email.as_deref())
        .bind(0) // not protected
        .execute(&app_state.pool)
        .await
        .expect("Failed to insert test user");

        let name: Option<String> = Some(Name().fake());
        let mobile = Some(format!(
            "9665{}",
            rand::thread_rng().gen_range(10000000..99999999)
        ));
        let email: Option<String> = Some(SafeEmail().fake());
        // Update user data
        let update_user_data = UpdateUserSchema {
            name: name.clone(),
            mobile: mobile.clone(),
            email: email.clone(),
            gender: Some("Male".to_string()),
            active: Some(true),
            protected: Some(false),
            role_id: None, // Assume role_id is not updated
        };

        let req = test::TestRequest::put()
            .uri(&format!("/admin/users/update/{}", user_id))
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .set_json(&update_user_data)
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "User updated");

        // Verify the user was updated in the database
        let updated_user = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", user_id)
            .fetch_one(&app_state.pool)
            .await
            .expect("Failed to fetch updated user");

        assert_eq!(updated_user.name, name.unwrap());
        assert_eq!(updated_user.mobile, mobile.unwrap());
        assert_eq!(updated_user.email, Some(email.unwrap()));
        assert_eq!(updated_user.gender, Some("Male".to_string()));
        assert_eq!(updated_user.active, 1);
        assert_eq!(updated_user.protected, 0);
    }
}
