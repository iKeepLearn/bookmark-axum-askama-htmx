use super::format_validation_errors;
use crate::error::Error;
use crate::utils::serde::parse_de_error::format_deserialization_error;
use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
};
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|_| Error::Validation("参数验证失败".to_string()))?;

        // 使用 serde_json 从 bytes 创建 Deserializer
        let mut deserializer = serde_json::Deserializer::from_slice(&bytes);

        // 使用 serde_path_to_error 进行反序列化并捕获路径
        match serde_path_to_error::deserialize::<_, T>(&mut deserializer) {
            Ok(value) => {
                value
                    .validate()
                    .map_err(|e| Error::ValidationFields(format_validation_errors(&e)))?;
                Ok(ValidatedJson(value))
            }
            Err(err) => {
                let friendly_msg =
                    format_deserialization_error(err.path(), &err.inner().to_string(), "body");
                Err(Error::Validation(friendly_msg))
            }
        }
    }
}
