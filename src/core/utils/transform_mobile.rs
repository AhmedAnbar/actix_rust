use regex::Regex;

use crate::config::CONFIG;

//COMM: Validates and transforms a mobile number based on the environment configuration.
///
//COMM: # Arguments
///
//COMM: * `mobile` - A string slice representing the mobile number to validate and transform.
///
//COMM: # Returns
///
//COMM: * `Result<String, String>` - A result containing the transformed mobile number as a string if valid,
//COMM:   or an error message string if validation fails.
pub fn validate_and_transform_mobile(mobile: &str) -> Result<String, String> {
    //COMM: Check if environment is production to apply transformation
    if CONFIG.env.eq_ignore_ascii_case("production") {
        //COMM: Define regex pattern for valid mobile numbers in production
        let re = Regex::new(r"^05\d{8}$").unwrap();

        //COMM: Check if mobile number matches the regex pattern
        if re.is_match(mobile) {
            //COMM: Transform and return the mobile number in production environment
            return Ok(format!("966{}", &mobile[1..]));
        } else {
            //COMM: Return error message for invalid mobile number in production environment
            return Err(
                "Invalid mobile number. It must start with 05 and be 10 digits long.".to_string(),
            );
        }
    }

    //COMM: Return the mobile number as-is for non-production environments
    Ok(format!("{}", &mobile))
}
