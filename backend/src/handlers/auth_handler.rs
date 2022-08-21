use actix_identity::Identity;
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};

use crate::api_error::ApiError;
use crate::models::AuthUser;

pub type LoggedUser = AuthUser;

impl FromRequest for LoggedUser {
    type Error = ApiError;
    type Future = Ready<Result<LoggedUser, ApiError>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
            if let Some(user_json) = identity.identity() {
                if let Ok(user) = serde_json::from_str(&user_json) {
                    return ok(user);
                }
            }
        }
        err(ApiError::new(401, "Invalid Request".to_string()))
    }
}
