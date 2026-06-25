use super::error::{ImageError, StorageError};
use bytes::Bytes;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::io::AsyncRead;

pub type StorageAsyncRead = Pin<Box<dyn AsyncRead + Send + Unpin>>;

pub trait Image: Send + Sync + 'static {
    fn convert_to_webp(
        &self,
        content: Bytes,
    ) -> impl Future<Output = Result<Bytes, ImageError>> + Send;
}

pub trait Storage: Send + Sync + 'static {
    fn join(&self, id: &str) -> PathBuf;

    fn get_path(&self, id: &str) -> impl Future<Output = Result<PathBuf, StorageError>> + Send;

    fn save(
        &self,
        path: impl AsRef<Path> + Send,
        content: Bytes,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    fn read(
        &self,
        path: impl AsRef<Path> + Send,
    ) -> impl Future<Output = Result<Vec<u8>, StorageError>> + Send;

    fn read_stream(
        &self,
        path: impl AsRef<Path> + Send,
    ) -> impl Future<Output = Result<StorageAsyncRead, StorageError>> + Send;
}
