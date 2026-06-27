use secrecy::{ExposeSecret, SecretString};

pub struct EncryptedPassword(SecretString);

impl EncryptedPassword {
    pub fn new(password: SecretString) -> Self {
        Self(password)
    }
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

pub struct Credentials {
    pub username: String,
    pub password: SecretString,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub username: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct UserToken {
    pub username: String,
    pub token: String,
    pub expire_time: u64,
}

impl UserInfo {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}
