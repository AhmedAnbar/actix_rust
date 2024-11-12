use std::future;

use actix_web::{FromRequest, HttpMessage};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::config::CONFIG;

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct Claims {
    pub exp: usize, //COMM: Expiration time as UNIX timestamp
    pub iat: usize, //COMM: Issued at time as UNIX timestamp
    pub id: String, //COMM: User ID
}

impl FromRequest for Claims {
    type Error = actix_web::Error;

    type Future = future::Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> std::future::Ready<Result<Claims, actix_web::Error>> {
        //COMM: Extract Claims from request extensions
        match req.extensions().get::<Claims>() {
            Some(claim) => future::ready(Ok(claim.clone())), // Return clone of Claims if found
            None => future::ready(Err(actix_web::error::ErrorBadRequest("Bad Claims"))), // Return BadRequest if Claims not found
        }
    }
}

//COMM: Encode JWT token with provided user ID and expiration duration
pub fn encode_jwt(id: String, expire: Duration) -> Result<String, jsonwebtoken::errors::Error> {
    if id.is_empty() {
        return Err(ErrorKind::InvalidSubject.into());
    }
    let now = Utc::now();
    let claims = Claims {
        exp: (now + expire).timestamp() as usize, //COMM: Calculate expiration time
        iat: now.timestamp() as usize,            //COMM: Set issued at time to current time
        id,                                       //COMM: Set user ID
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt.secret.as_ref()), //COMM: Encode JWT using secret key
    )
}

//COMM: Decode JWT token and validate with secret key
pub fn decode_jwt(token: String) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    if token.is_empty() {
        return Err(ErrorKind::InvalidSubject.into());
    }
    let claim_data: Result<TokenData<Claims>, jsonwebtoken::errors::Error> = decode(
        &token,
        &DecodingKey::from_secret(CONFIG.jwt.secret.as_ref()), //COMM: Decode JWT using secret key
        &Validation::default(),
    );

    match claim_data {
        Ok(claims) => Ok(claims),
        Err(_) => Err(ErrorKind::InvalidToken.into()),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_create_and_decoded_valid_token() {
        let user_id = "a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b".to_string();

        let token = encode_jwt(user_id.clone(), Duration::hours(1)).unwrap();
        let decoded_user_id = decode_jwt(token).unwrap();

        assert_eq!(decoded_user_id.claims.id, user_id);
    }

    #[test]
    fn test_create_token_with_empty_user_id() {
        let user_id = "".to_string();

        let result = encode_jwt(user_id, Duration::hours(1));

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().into_kind(),
            jsonwebtoken::errors::ErrorKind::InvalidSubject
        )
    }

    #[test]
    fn test_decoded_invalid_token() {
        let invalid_token = "invalid-token".to_string();

        let result = decode_jwt(invalid_token);
        assert!(result.is_err());
        assert_eq!(result.clone().unwrap_err(), ErrorKind::InvalidToken.into());
    }

    #[test]
    fn test_decode_expired_token() {
        let user_id = "a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b".to_string();
        let expired_token = encode_jwt(user_id, Duration::hours(-1)).unwrap();

        let result = decode_jwt(expired_token);

        assert!(result.is_err());
        assert_eq!(result.clone().unwrap_err(), ErrorKind::InvalidToken.into());
    }
}
