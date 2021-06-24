use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, Error};
use actix_web_httpauth::extractors::basic::BasicAuth;

use crate::config;
// In this example validator returns immediately, but since it is required to return
// anything that implements `IntoFuture` trait, it can be extended to query database or to
// do something else in a async manner.
pub async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, Error> {
    if let Some(api_key) = config::api_key() {
        if let Some(password) = credentials.password() {
            if password.eq(&api_key) {
                return Ok(req);
            }
        }
        Err(ErrorUnauthorized("API key incorrect"))
    } else {
        Ok(req)
    }
}
