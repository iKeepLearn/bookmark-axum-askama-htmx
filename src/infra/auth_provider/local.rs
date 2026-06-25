use crate::domain::user::error::AuthError;
use crate::domain::user::models::EncryptedPassword;
use crate::domain::user::traits::AuthProvider;
use anyhow::Context;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use secrecy::{ExposeSecret, SecretString};
use tokio::task::spawn_blocking;

#[derive(Default, Debug, Clone)]
pub struct LocalAuthProvider {}

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
}
