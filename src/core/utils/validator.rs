use crate::schema::response::api_response_error::{ApiResponseError, ValidationErrorDetail};
use validator::Validate;

//COMM: Validates a request object using the `validator` crate and converts validation errors
//COMM: into an `ApiResponseError` containing detailed validation error messages.
///
//COMM: # Arguments
///
//COMM: * `data` - A reference to the data structure implementing `Validate` trait.
///
//COMM: # Returns
///
//COMM: * `Result<(), ApiResponseError>` - `Ok(())` if validation succeeds, or an `ApiResponseError`
//COMM:   containing validation error details if validation fails.
pub fn validate_request<T: Validate + 'static>(data: &T) -> Result<(), ApiResponseError> {
    //COMM: Validate the data using the `validator` crate
    if let Err(err) = data.validate() {
        //COMM: Convert validation errors into `ValidationErrorDetail` structs
        let validation_errors: Vec<ValidationErrorDetail> = err
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| ValidationErrorDetail {
                    field: field.to_string(),
                    error: error
                        .message
                        .clone()
                        .unwrap_or_else(|| "Validation error".into())
                        .to_string(),
                })
            })
            .collect();

        //COMM: Return an `ApiResponseError` with status code 400 and validation error details
        return Err(ApiResponseError::new(
            400,
            "Validation Error".to_string(),
            Some(validation_errors),
        ));
    }

    //COMM: Return Ok(()) if validation succeeds
    Ok(())
}
