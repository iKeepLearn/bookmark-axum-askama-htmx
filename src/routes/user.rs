use super::PageResult;
use super::extractor::i18n::Locale;
use crate::domain::user::models::{Credentials, UserInfo, UserToken};
use crate::domain::user::services::UserService;
use crate::infra::auth_provider::local::LocalAuthProvider;
use crate::infra::database::user::PgUserRepository;
use crate::utils::askama::filters;
use crate::utils::i18n::t_for;
use askama::Template;
use axum::Form;
use axum::extract::State;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use tracing::error;

const FLASH_KEY: &str = "flash_messages";

#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct LoginTemplate {
    flash_message: String,
    errors: LoginErrors,
    username: String,
}

#[derive(Template)]
#[template(path = "pages/change_password.html")]
pub struct ChangePwdTemplate {
    msg: String,
}

#[derive(Default)]
struct LoginErrors {
    username: Option<String>,
    password: Option<String>,
    general: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: SecretString,
}

#[derive(Deserialize)]
pub struct ChangePwdForm {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SessionUser {
    pub username: String,
    pub role: String,
    pub is_admin: bool,
}

impl From<UserInfo> for SessionUser {
    fn from(value: UserInfo) -> Self {
        SessionUser {
            username: value.username,
            role: value.role.clone(),
            is_admin: &value.role == "admin",
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApiUser {
    pub username: String,
    pub token: String,
}

impl From<UserToken> for ApiUser {
    fn from(value: UserToken) -> Self {
        ApiUser {
            username: value.username,
            token: value.token,
        }
    }
}

pub async fn login_form(session: Session) -> PageResult<LoginTemplate, &'static str> {
    let flash_message: String = session
        .remove(FLASH_KEY) // remove = 取出并删除，flash消息只显示一次
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    PageResult::RenderTemplate(LoginTemplate {
        flash_message,
        errors: LoginErrors::default(),
        username: String::new(),
    })
}

pub async fn login_submit(
    session: Session,
    locale: Locale,
    State(service): State<UserService<PgUserRepository>>,
    State(auth_provider): State<LocalAuthProvider>,
    Form(form): Form<LoginForm>,
) -> PageResult<LoginTemplate, &'static str> {
    let mut errors = LoginErrors::default();

    if form.username.trim().is_empty() {
        errors.username = Some(t_for(&locale.lang, "username_required"));
    }
    if form.password.expose_secret().is_empty() {
        errors.password = Some(t_for(&locale.lang, "password_required"));
    }

    if errors.username.is_none() && errors.password.is_none() {
        let credentials = Credentials {
            username: form.username.clone(),
            password: form.password,
        };
        match service.authenticate(&credentials, &auth_provider).await {
            Ok(user) => {
                let session_user: SessionUser = user.into();
                session.insert("user", session_user).await.unwrap();

                return PageResult::Redirect("/");
            }
            Err(err) => {
                error!("Failed to authenticate: {}", err);
                errors.general = Some(t_for(&locale.lang, "invalid_credentials"))
            }
        }
    }

    PageResult::RenderTemplate(LoginTemplate {
        flash_message: String::new(),
        errors,
        username: form.username,
    })
}

pub async fn logout(session: Session) -> PageResult<LoginTemplate, &'static str> {
    session.flush().await.unwrap(); // 清空并让cookie失效
    PageResult::Redirect("/login")
}

pub async fn changepwd_form() -> PageResult<ChangePwdTemplate, &'static str> {
    PageResult::RenderTemplate(ChangePwdTemplate { msg: String::new() })
}

pub async fn changepwd_submit(
    session: Session,
    user: SessionUser,
    locale: Locale,
    State(service): State<UserService<PgUserRepository>>,
    State(auth_provider): State<LocalAuthProvider>,
    Form(form): Form<ChangePwdForm>,
) -> PageResult<ChangePwdTemplate, &'static str> {
    // 校验字段
    if form.current_password.expose_secret().is_empty() {
        return PageResult::RenderTemplate(ChangePwdTemplate {
            msg: t_for(&locale.lang, "current_password_required"),
        });
    }
    if form.new_password.expose_secret().is_empty() {
        return PageResult::RenderTemplate(ChangePwdTemplate {
            msg: t_for(&locale.lang, "new_password_required"),
        });
    }
    if form.new_password_check.expose_secret().is_empty() {
        return PageResult::RenderTemplate(ChangePwdTemplate {
            msg: t_for(&locale.lang, "confirm_password_required"),
        });
    }
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        return PageResult::RenderTemplate(ChangePwdTemplate {
            msg: t_for(&locale.lang, "password_mismatch"),
        });
    }

    let credentials = Credentials {
        username: user.username,
        password: form.current_password,
    };
    match service
        .change_password(&credentials, form.new_password, &auth_provider)
        .await
    {
        Ok(_) => {
            // 修改成功，退出登录（安全起见）
            session.flush().await.unwrap();

            // 跳转到登录页
            PageResult::Redirect("/login")
        }
        Err(_) => PageResult::RenderTemplate(ChangePwdTemplate {
            msg: t_for(&locale.lang, "current_password_wrong"),
        }),
    }
}

#[derive(Template)]
#[template(path = "pages/api_token.html")]
pub struct TokenTemplate {
    flash_message: String,
    errors: LoginErrors,
    username: String,
}

pub async fn token_form(session: Session) -> PageResult<TokenTemplate, &'static str> {
    let flash_message: String = session
        .remove(FLASH_KEY) // remove = 取出并删除，flash消息只显示一次
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    PageResult::RenderTemplate(TokenTemplate {
        flash_message,
        errors: LoginErrors::default(),
        username: String::new(),
    })
}

pub async fn get_token_submit(
    locale: Locale,
    State(service): State<UserService<PgUserRepository>>,
    State(auth_provider): State<LocalAuthProvider>,
    Form(form): Form<LoginForm>,
) -> PageResult<TokenTemplate, &'static str> {
    let mut errors = LoginErrors::default();

    if form.username.trim().is_empty() {
        errors.username = Some(t_for(&locale.lang, "username_required"));
    }
    if form.password.expose_secret().is_empty() {
        errors.password = Some(t_for(&locale.lang, "password_required"));
    }

    if errors.username.is_none() && errors.password.is_none() {
        let credentials = Credentials {
            username: form.username.clone(),
            password: form.password,
        };
        match service.get_api_token(&credentials, &auth_provider).await {
            Ok(user) => {
                return PageResult::RenderTemplate(TokenTemplate {
                    flash_message: user.token,
                    errors,
                    username: form.username,
                });
            }
            Err(err) => {
                error!("Failed to authenticate: {}", err);
                errors.general = Some(t_for(&locale.lang, "invalid_permission"));
            }
        }
    }

    PageResult::RenderTemplate(TokenTemplate {
        flash_message: String::new(),
        errors,
        username: form.username,
    })
}
