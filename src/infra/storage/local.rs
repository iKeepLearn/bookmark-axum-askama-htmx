use bytes::Bytes;

use crate::domain::image::error::StorageError;
use crate::domain::image::traits::{Storage, StorageAsyncRead};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LocalStorage {
    root: PathBuf,
}

impl LocalStorage {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let path = root.as_ref();
        if !path.exists() {
            std::fs::create_dir_all(path).expect("failed to create local storage root");
        }
        Self {
            root: path.to_path_buf(),
        }
    }
}

impl Storage for LocalStorage {
    fn join(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }
    async fn get_path(&self, id: &str) -> Result<PathBuf, StorageError> {
        let file_path = self.join(id);

        let canonicalized_path = match tokio::fs::canonicalize(&file_path).await {
            Ok(path) => path,
            Err(_) => return Err(StorageError::NotFound),
        };

        if !canonicalized_path.starts_with(&self.root) {
            return Err(StorageError::NotFound);
        }

        Ok(canonicalized_path)
    }

    async fn save(&self, path: impl AsRef<Path>, content: Bytes) -> Result<(), StorageError> {
        tokio::fs::write(path.as_ref(), content)
            .await
            .map_err(|e| StorageError::Unkown(e.into()))
    }

    async fn read(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, StorageError> {
        let content = tokio::fs::read(path.as_ref())
            .await
            .map_err(|_| StorageError::NotFound)?;
        Ok(content)
    }

    async fn read_stream(
        &self,
        path: impl AsRef<Path> + Send,
    ) -> Result<StorageAsyncRead, StorageError> {
        let file = tokio::fs::File::open(path.as_ref())
            .await
            .map_err(|_| StorageError::NotFound)?;
        Ok(Box::pin(file))
    }
}
