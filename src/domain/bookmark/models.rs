use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CategoryWithBookmarkCount {
    pub id: i32,
    pub name: String,
    pub bookmark_count: i64,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub tags: Vec<Tag>,
    pub category_id: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BookmarkAdd {
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub desc: String,
    pub tags: Vec<i32>,
    pub new_tags: Vec<String>,
    pub category_id: i32,
}

#[derive(Debug, Clone)]
pub struct BookmarkBatchAdd {
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub desc: String,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct BookmarkUpdate {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub desc: String,
    pub tags: Vec<i32>,
    pub new_tags: Vec<String>,
    pub category_id: i32,
}

#[derive(Debug, Clone)]
pub struct QueryArgs {
    pub category_id: Option<i32>,
    pub tags: Option<Vec<i32>>,
    pub keyword: Option<String>,
    pub page: i32,
    pub page_size: i32,
}
