use secrecy::SecretString;

use super::error::{AuthError, UserError};
use super::models::{EncryptedPassword, UserInfo};

pub trait UserRepository: Send + Sync + 'static {
    fn get_user_by_username(
        &self,
        username: &str,
    ) -> impl Future<Output = Result<UserInfo, UserError>> + Send;

    fn get_user_password(
        &self,
        username: &str,
    ) -> impl Future<Output = Result<EncryptedPassword, UserError>> + Send;

    fn change_password(
        &self,
        username: &str,
        new_password: &EncryptedPassword,
    ) -> impl Future<Output = Result<(), UserError>> + Send;
}

pub trait AuthProvider: Send + Sync + 'static {
    fn verify_password(
        &self,
        password: EncryptedPassword,
        password_candidate: SecretString,
    ) -> impl Future<Output = Result<(), AuthError>> + Send;
    fn encrypt_password(
        &self,
        password: SecretString,
    ) -> impl Future<Output = Result<EncryptedPassword, AuthError>> + Send;
}
