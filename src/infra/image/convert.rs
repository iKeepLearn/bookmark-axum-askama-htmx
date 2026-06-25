use crate::domain::image::error::ImageError;
use crate::domain::image::traits::Image;
use bytes::Bytes;
use tokio::task::spawn_blocking;
use webp::Encoder;

#[derive(Debug, Clone)]
pub struct ImageConverter {
    quality: f32,
}

impl ImageConverter {
    pub fn new(quality: f32) -> Self {
        Self { quality }
    }
}

impl Image for ImageConverter {
    async fn convert_to_webp(&self, content: Bytes) -> Result<Bytes, ImageError> {
        let quality = self.quality;
        let webp: Bytes = spawn_blocking(move || -> Result<Bytes, ImageError> {
            let img = image::load_from_memory(&content).map_err(|_| ImageError::InvalidFormat)?;
            let encoder = Encoder::from_image(&img).map_err(|_| ImageError::EncodeFailed)?;
            Ok(encoder.encode(quality).to_vec().into())
        })
        .await
        .map_err(|_| ImageError::EncodeFailed)??;
        Ok(webp)
    }
}
