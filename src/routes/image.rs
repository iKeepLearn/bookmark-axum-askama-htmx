use crate::Error;
use crate::domain::image::services::ImageService;
use crate::infra::image::convert::ImageConverter;
use crate::infra::storage::local::LocalStorage;
use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path as AxumPath, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use http::header;
use serde_json::json;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[tracing::instrument(name = "upload image", skip(multipart))]
#[axum::debug_handler]
pub async fn upload_image(
    State(service): State<ImageService<ImageConverter, LocalStorage>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, Error> {
    let mut image_data = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| Error::bad_request("Failed to parse multipart"))?
    {
        if field.name() == Some("image") {
            let is_image = field
                .content_type()
                .map(|mime| mime.starts_with("image/"))
                .unwrap_or(false);

            if !is_image {
                return Err(Error::bad_request(
                    "Content-Type header is missing or uploaded file is not an image",
                ));
            }

            let data = field
                .bytes()
                .await
                .map_err(|_| Error::internal("Failed to read file"))?;

            image_data = Some(data);
            break;
        }
    }

    let file_buffer = match image_data {
        Some(data) => data,
        None => {
            return Err(Error::bad_request("Missing 'image' field"));
        }
    };
    let uuid = Uuid::new_v4().to_string();
    let filename = format!("{}.{}", uuid, "webp");
    let _ = service
        .save_image(&filename, file_buffer)
        .await
        .map_err(|_| Error::internal("failed to save image"))?;

    let public_url = format!("images/{}", filename);

    Ok((
        StatusCode::OK,
        Json(json!({
            "image_url": public_url
        })),
    ))
}

pub async fn get_image(
    State(service): State<ImageService<ImageConverter, LocalStorage>>,
    AxumPath(id): AxumPath<String>,
) -> Result<impl IntoResponse, Error> {
    let reader = service
        .get_image(&id)
        .await
        .map_err(|_| Error::internal("failed to read image"))?;
    let mime_type = "image/webp".to_string();

    let stream = ReaderStream::new(reader);
    let body = Body::from_stream(stream);

    let response = Response::builder()
        .header(header::CONTENT_TYPE, &mime_type)
        .body(body)
        .map_err(|_| Error::internal("Failed to build response"))?;

    Ok(response)
}
