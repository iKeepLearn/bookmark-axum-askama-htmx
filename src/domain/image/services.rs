use super::error::ImageError;
use super::traits::{Image, Storage, StorageAsyncRead};
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct ImageService<I: Image, S: Storage> {
    pub image: I,
    pub storage: S,
}

impl<I: Image, S: Storage> ImageService<I, S> {
    pub fn new(image: I, storage: S) -> Self {
        Self { image, storage }
    }

    pub async fn get_image(&self, id: &str) -> Result<StorageAsyncRead, ImageError> {
        let path = self
            .storage
            .get_path(id)
            .await
            .map_err(|e| ImageError::Unkown(e.into()))?;

        let content = self
            .storage
            .read_stream(&path)
            .await
            .map_err(|e| ImageError::Unkown(e.into()))?;

        Ok(content)
    }

    pub async fn save_image(&self, id: &str, content: Bytes) -> Result<(), ImageError> {
        let path = self.storage.join(id);
        let webp = self.image.convert_to_webp(content).await?;
        self.storage
            .save(&path, webp)
            .await
            .map_err(|e| ImageError::Unkown(e.into()))?;
        Ok(())
    }
}
