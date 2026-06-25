use super::error::BookmarkError;
use super::models::{
    Bookmark, BookmarkAdd, BookmarkBatchAdd, BookmarkUpdate, Category, CategoryWithBookmarkCount,
    QueryArgs, Tag,
};
use super::traits::BookmarkRepository;

#[derive(Debug, Clone)]
pub struct BookmarkService<R: BookmarkRepository> {
    pub repo: R,
}

impl<R: BookmarkRepository> BookmarkService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn add_bookmark(&self, bookmark: BookmarkAdd) -> Result<(), BookmarkError> {
        self.repo.add_bookmark(bookmark).await
    }

    pub async fn update_bookmark(&self, bookmark: BookmarkUpdate) -> Result<(), BookmarkError> {
        self.repo.update_bookmark(bookmark).await
    }

    pub async fn delete_bookmark(&self, id: i32) -> Result<(), BookmarkError> {
        self.repo.delete_bookmark(id).await
    }

    pub async fn get_bookmark_by_id(&self, id: i32) -> Result<Bookmark, BookmarkError> {
        self.repo.get_bookmark_by_id(id).await
    }

    pub async fn get_bookmarks_by_query(
        &self,
        query: &QueryArgs,
    ) -> Result<Vec<Bookmark>, BookmarkError> {
        self.repo.get_bookmarks_by_query(query).await
    }

    pub async fn get_categories(&self) -> Result<Vec<Category>, BookmarkError> {
        self.repo.get_categories().await
    }

    pub async fn get_categories_with_bookmark_count(
        &self,
    ) -> Result<Vec<CategoryWithBookmarkCount>, BookmarkError> {
        self.repo.get_categories_with_bookmark_count().await
    }

    pub async fn get_tags(&self) -> Result<Vec<Tag>, BookmarkError> {
        self.repo.get_tags().await
    }

    pub async fn batch_add_bookmarks(
        &self,
        categories: Vec<String>,
        bookmarks: Vec<BookmarkBatchAdd>,
    ) -> Result<(), BookmarkError> {
        let mut all_category_names: Vec<String> = categories
            .iter()
            .map(|s| s.as_str())
            .chain(bookmarks.iter().map(|b| b.category.as_str()))
            .map(|s| s.to_string())
            .collect();
        all_category_names.sort_unstable();
        all_category_names.dedup();
        self.repo
            .batch_add_bookmarks(all_category_names, bookmarks)
            .await
    }

    pub async fn get_categories_tags(&self) -> (Vec<Category>, Vec<Tag>) {
        let (categories_res, tags_res) = tokio::join!(self.get_categories(), self.get_tags());

        let categories = categories_res.unwrap_or_default();
        let tags = tags_res.unwrap_or_default();

        (categories, tags)
    }

    pub async fn get_bookmark_tag_ids(&self, id: i32) -> Result<Vec<i32>, BookmarkError> {
        self.repo.get_bookmark_tag_ids(id).await
    }
}
