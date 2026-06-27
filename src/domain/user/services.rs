use super::error::{AuthError, UserError};
use super::models::{Credentials, UserInfo, UserToken};
use super::traits::{AuthProvider, UserRepository};
use secrecy::SecretString;

#[derive(Debug, Clone)]
pub struct UserService<R: UserRepository> {
    pub repo: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn authenticate(
        &self,
        credentials: &Credentials,
        auth_provider: &impl AuthProvider,
    ) -> Result<UserInfo, AuthError> {
        match self.repo.get_user_password(&credentials.username).await {
            Ok(password) => {
                if auth_provider
                    .verify_password(password, credentials.password.clone())
                    .await
                    .is_err()
                {
                    return Err(AuthError::InvalidCredentials(anyhow::anyhow!(
                        "Invalid password."
                    )));
                }

                let user = self
                    .repo
                    .get_user_by_username(&credentials.username)
                    .await
                    .map_err(|e| AuthError::UnexpectedError(e.into()))?;

                Ok(user)
            }
            Err(err) => Err(AuthError::InvalidCredentials(err.into())),
        }
    }

    pub async fn change_password(
        &self,
        credentials: &Credentials,
        new_password: SecretString,
        auth_provider: &impl AuthProvider,
    ) -> Result<(), UserError> {
        let user = self
            .authenticate(credentials, auth_provider)
            .await
            .map_err(|_| UserError::InvalidPassword)?;

        let encrypted_password = auth_provider
            .encrypt_password(new_password)
            .await
            .map_err(|e| UserError::Unkown(e.into()))?;
        self.repo
            .change_password(&user.username, &encrypted_password)
            .await
            .map_err(|e| UserError::Unkown(e.into()))
    }

    pub async fn get_api_token<T: AuthProvider>(
        &self,
        credentials: &Credentials,
        token_provider: &T,
    ) -> Result<UserToken, AuthError> {
        let user = self
            .authenticate(credentials, token_provider)
            .await
            .map_err(|_| AuthError::InvalidCredentials(anyhow::anyhow!("Invalid credentials")))?;
        if user.is_admin() {
            Ok(token_provider.issue_token_with_username(&user.username)?)
        } else {
            Err(AuthError::InvalidPermission(anyhow::anyhow!(
                "Invalid permission"
            )))
        }
    }
}
