use jwt_simple::prelude::*;
use poem::Request;
use poem_openapi::{auth::Bearer, SecurityScheme};

use crate::config::SETTINGS;

/// ApiKey authorization
#[derive(SecurityScheme)]
#[oai(
    type = "bearer",
    key_name = "Authorization",
    in = "header",
    checker = "api_checker"
)]
pub struct JWTAuthorization(pub String);

async fn api_checker(_: &Request, api_key: Bearer) -> Option<String> {
    let key = HS256Key::from_bytes(SETTINGS.secret.as_bytes());
    let ret = key
        .verify_token::<JWTClaims<NoCustomClaims>>(api_key.token.as_str(), None)
        .ok()?;
    ret.subject
}
