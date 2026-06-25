use crate::domain::user::error::UserError;
use crate::domain::user::models::{EncryptedPassword, UserInfo as DUserInfo};
use crate::domain::user::traits::UserRepository;
use sqlx::{FromRow, PgPool};
use tracing::error;

#[derive(Debug, FromRow)]
pub struct User {
    pub username: String,
    pub role: String,
}

impl From<User> for DUserInfo {
    fn from(user: User) -> Self {
        DUserInfo {
            username: user.username,
            role: user.role,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PgUserRepository {
    pub pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        PgUserRepository { pool }
    }
}

impl UserRepository for PgUserRepository {
    async fn get_user_by_username(&self, username: &str) -> Result<DUserInfo, UserError> {
        match sqlx::query_as::<_, User>(
            r#"
            SELECT username, role FROM users WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        {
            Ok(user) => Ok(user.into()),
            Err(e) => {
                error!("get_user_by_username sqlx error {}", e);
                Err(UserError::NotFound)
            }
        }
    }

    async fn get_user_password(&self, username: &str) -> Result<EncryptedPassword, UserError> {
        match sqlx::query_scalar::<_, String>(
            r#"
            SELECT password FROM users WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        {
            Ok(password) => Ok(EncryptedPassword::new(password.into())),
            Err(e) => {
                error!("get_user_password sqlx error {}", e);
                Err(UserError::NotFound)
            }
        }
    }

    async fn change_password(
        &self,
        username: &str,
        new_password: &EncryptedPassword,
    ) -> Result<(), UserError> {
        match sqlx::query(
            r#"
            UPDATE users SET password = $1 WHERE username = $2
            "#,
        )
        .bind(new_password.expose_secret())
        .bind(username)
        .execute(&self.pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("change_password sqlx error {}", e);
                Err(UserError::Unkown(e.into()))
            }
        }
    }
}
