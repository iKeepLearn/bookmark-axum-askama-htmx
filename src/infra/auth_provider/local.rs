use crate::domain::user::error::AuthError;
use crate::domain::user::models::{EncryptedPassword, UserToken};
use crate::domain::user::traits::AuthProvider;
use anyhow::Context;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use chrono::{TimeDelta, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;

#[derive(Default, Debug, Clone)]
pub struct LocalAuthProvider {
    pub secret_key: SecretString,
    pub expire_time: TimeDelta,
}

impl LocalAuthProvider {
    pub fn new(secret_key: SecretString, expire_time: impl Into<String>) -> Self {
        let expire_time = humantime::parse_duration(&expire_time.into())
            .context("Failed to parse expire time.")
            .unwrap_or_default();
        let default_expire_time = TimeDelta::try_days(2).unwrap_or_default();
        let expire_time =
            TimeDelta::try_seconds(expire_time.as_secs() as i64).unwrap_or(default_expire_time);

        Self {
            secret_key,
            expire_time,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPlayload {
    pub sub: String,
    pub exp: u64,
}

impl AuthProvider for LocalAuthProvider {
    async fn verify_password(
        &self,
        password: EncryptedPassword,
        password_candidate: SecretString,
    ) -> Result<(), AuthError> {
        let _ = spawn_blocking(move || {
            let password_hash = PasswordHash::new(password.expose_secret())
                .context("Failed to parse hash in PHC string format.")?;
            Argon2::default()
                .verify_password(
                    password_candidate.expose_secret().as_bytes(),
                    &password_hash,
                )
                .context("Invalid password.")
                .map_err(AuthError::InvalidCredentials)
        })
        .await
        .context("Failed to verify password.")??;
        Ok(())
    }

    async fn encrypt_password(
        &self,
        password: SecretString,
    ) -> Result<EncryptedPassword, AuthError> {
        let encrypted_password = spawn_blocking(move || -> Result<EncryptedPassword, AuthError> {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None).unwrap(),
            )
            .hash_password(password.expose_secret().as_bytes(), &salt)
            .map_err(|e| AuthError::InvalidCredentials(e.into()))?
            .to_string();
            Ok(EncryptedPassword::new(password_hash.into()))
        })
        .await
        .map_err(|_| {
            AuthError::UnexpectedError(anyhow::anyhow!("Failed to encrypt password."))
        })??;
        Ok(encrypted_password)
    }

    fn issue_token_with_username(&self, username: &str) -> Result<UserToken, AuthError> {
        let token_playload = TokenPlayload {
            sub: username.to_string(),
            exp: (Utc::now() + self.expire_time).timestamp() as u64,
        };

        let token = jsonwebtoken::encode(
            &Header::default(),
            &token_playload,
            &EncodingKey::from_secret(self.secret_key.expose_secret().as_bytes()),
        )
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        let expire_time = self.expire_time.num_milliseconds() as u64;
        Ok(UserToken {
            username: username.to_string(),
            token,
            expire_time,
        })
    }

    fn verify_token(&self, token: &str) -> Result<UserToken, AuthError> {
        let decode_token = jsonwebtoken::decode::<TokenPlayload>(
            token,
            &DecodingKey::from_secret(self.secret_key.expose_secret().as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AuthError::InvalidCredentials(e.into()))?;
        Ok(UserToken {
            username: decode_token.claims.sub,
            token: token.to_string(),
            expire_time: decode_token.claims.exp as u64,
        })
    }
}
