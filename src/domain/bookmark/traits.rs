use super::error::BookmarkError;
use super::models::{
    Bookmark, BookmarkAdd, BookmarkBatchAdd, BookmarkUpdate, Category, CategoryWithBookmarkCount,
    QueryArgs, Tag,
};

pub trait BookmarkRepository: Send + Sync + 'static {
    fn get_bookmarks_by_query(
        &self,
        query: &QueryArgs,
    ) -> impl Future<Output = Result<Vec<Bookmark>, BookmarkError>> + Send;

    fn get_bookmark_by_id(
        &self,
        id: i32,
    ) -> impl Future<Output = Result<Bookmark, BookmarkError>> + Send;

    fn add_bookmark(
        &self,
        bookmark: BookmarkAdd,
    ) -> impl Future<Output = Result<Bookmark, BookmarkError>> + Send;

    fn update_bookmark(
        &self,
        bookmark: BookmarkUpdate,
    ) -> impl Future<Output = Result<(), BookmarkError>> + Send;

    fn delete_bookmark(&self, id: i32) -> impl Future<Output = Result<(), BookmarkError>> + Send;

    fn get_categories(&self) -> impl Future<Output = Result<Vec<Category>, BookmarkError>> + Send;

    fn get_categories_with_bookmark_count(
        &self,
    ) -> impl Future<Output = Result<Vec<CategoryWithBookmarkCount>, BookmarkError>> + Send;

    fn get_tags(&self) -> impl Future<Output = Result<Vec<Tag>, BookmarkError>> + Send;

    fn batch_add_bookmarks(
        &self,
        categories: Vec<String>,
        bookmarks: Vec<BookmarkBatchAdd>,
    ) -> impl Future<Output = Result<(), BookmarkError>> + Send;

    fn get_bookmark_tag_ids(
        &self,
        id: i32,
    ) -> impl Future<Output = Result<Vec<i32>, BookmarkError>> + Send;
}
