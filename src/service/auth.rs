use super::{ApiTags, AppState};
use crate::{config::SETTINGS, oai_schema};
use crate::{errors::NeoiotError, repository::Repository};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use jwt_simple::prelude::*;
use poem::web::Data;
use poem::Result;
use poem_openapi::{payload::Json, OpenApi};

pub struct AuthService;

#[OpenApi(prefix_path = "/auth/token", tag = "ApiTags::Auth")]
impl AuthService {
    /// 获取Token
    #[oai(path = "/obtain", method = "post")]
    async fn obtain_token(
        &self,
        state: Data<&AppState>,
        data: Json<oai_schema::Login>,
    ) -> Result<Json<oai_schema::TokenResponse>> {
        let account = state
            .repo
            .get_account_by_email(&data.email)
            .await
            .map_err(|_| NeoiotError::AuthenticateError)?;
        if !verify_password(&data.password, &account.password) {
            return Err(NeoiotError::AuthenticateError.into());
        }
        let claims = Claims::create(Duration::from_days(1)).with_subject(account.id);
        let key = HS256Key::from_bytes(SETTINGS.core.secret.as_bytes());
        let token = key.authenticate(claims)?;
        state.repo.after_account_logined(&data.email).await?;
        Ok(Json(oai_schema::TokenResponse { token }))
    }
}

fn verify_password(password: &str, hash: &str) -> bool {
    let argon2 = Argon2::default();
    let hashed = PasswordHash::new(hash).unwrap();
    argon2.verify_password(password.as_bytes(), &hashed).is_ok()
}
