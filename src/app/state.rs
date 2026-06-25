use super::config::{Settings, get_connect_pool};
use crate::domain::bookmark::services::BookmarkService;
use crate::domain::image::services::ImageService;
use crate::domain::user::services::UserService;
use crate::infra::auth_provider::local::LocalAuthProvider;
use crate::infra::database::bookmark::PgBookmarkRepository;
use crate::infra::database::user::PgUserRepository;
use crate::infra::image::convert::ImageConverter;
use crate::infra::storage::local::LocalStorage;
use axum::extract::FromRef;
use sqlx::{Pool, Postgres};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StaticDirectory(pub String);
#[derive(Debug, Clone)]
pub struct UploadDirectory(pub String);

#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub static_directory: StaticDirectory,
    pub bookmark_service: BookmarkService<PgBookmarkRepository>,
    pub user_service: UserService<PgUserRepository>,
    pub image_service: ImageService<ImageConverter, LocalStorage>,
    pub auth_provider: LocalAuthProvider,
}

impl AppState {
    pub fn new(config: Settings) -> Self {
        let pool = get_connect_pool(&config.database);
        let static_directory = StaticDirectory(config.application.static_directory);
        let upload_driectory = PathBuf::from(config.application.upload_directory);

        let bookmark_repository = PgBookmarkRepository::new(pool.clone());
        let bookmark_service = BookmarkService::new(bookmark_repository);

        let user_repository = PgUserRepository::new(pool.clone());
        let user_service = UserService::new(user_repository);

        let auth_provider = LocalAuthProvider::default();
        let image = ImageConverter::new(config.application.image_quality);
        let image_service = ImageService::new(image, LocalStorage::new(upload_driectory));

        Self {
            pool,
            static_directory,
            bookmark_service,
            user_service,
            image_service,
            auth_provider,
        }
    }
}
