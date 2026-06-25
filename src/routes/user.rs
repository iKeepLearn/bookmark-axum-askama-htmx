use super::PageResult;
use crate::domain::user::models::{Credentials, UserInfo};
use crate::domain::user::services::UserService;
use crate::infra::auth_provider::local::LocalAuthProvider;
use crate::infra::database::user::PgUserRepository;
use askama::Template;
use axum::Form;
use axum::extract::State;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

const FLASH_KEY: &str = "flash_messages";

#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct LoginTemplate {
    flash_messages: Vec<String>,
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

pub async fn login_form(session: Session) -> PageResult<LoginTemplate, &'static str> {
    let flash_messages: Vec<String> = session
        .remove(FLASH_KEY) // remove = 取出并删除，flash消息只显示一次
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

    PageResult::RenderTemplate(LoginTemplate {
        flash_messages,
        errors: LoginErrors::default(),
        username: String::new(),
    })
}

pub async fn login_submit(
    session: Session,
    State(service): State<UserService<PgUserRepository>>,
    State(auth_provider): State<LocalAuthProvider>,
    Form(form): Form<LoginForm>,
) -> PageResult<LoginTemplate, &'static str> {
    let mut errors = LoginErrors::default();

    if form.username.trim().is_empty() {
        errors.username = Some("用户名不能为空".into());
    }
    if form.password.expose_secret().is_empty() {
        errors.password = Some("密码不能为空".into());
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
            Err(_) => errors.general = Some("用户名或密码错误".into()),
        }
    }

    PageResult::RenderTemplate(LoginTemplate {
        flash_messages: vec![],
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
    State(service): State<UserService<PgUserRepository>>,
    State(auth_provider): State<LocalAuthProvider>,
    Form(form): Form<ChangePwdForm>,
) -> PageResult<ChangePwdTemplate, &'static str> {
    // 校验字段
    if form.current_password.expose_secret().is_empty() {
        return PageResult::RenderTemplate(ChangePwdTemplate {
            msg: "当前密码不能为空".into(),
        });
    }
    if form.new_password.expose_secret().is_empty() {
        return PageResult::RenderTemplate(ChangePwdTemplate {
            msg: "新密码不能为空".into(),
        });
    }
    if form.new_password_check.expose_secret().is_empty() {
        return PageResult::RenderTemplate(ChangePwdTemplate {
            msg: "请确认新密码".into(),
        });
    }
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        return PageResult::RenderTemplate(ChangePwdTemplate {
            msg: "两次输入的新密码不一致".into(),
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
            msg: "当前密码错误".into(),
        }),
    }
}
