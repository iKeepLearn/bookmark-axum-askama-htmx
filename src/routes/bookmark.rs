use crate::domain::bookmark::models::{
    BookmarkAdd, BookmarkBatchAdd, BookmarkUpdate, Category, Tag,
};
use crate::domain::bookmark::services::BookmarkService;
use crate::infra::database::bookmark::PgBookmarkRepository;
use crate::utils::{e500, render_template};
use askama::Template;
use axum::Json;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Redirect};
use axum_extra::extract::Form;
use serde::Deserialize;

#[derive(Debug, Template)]
#[template(path = "pages/add_bookmark.html")]
pub struct AddBookmarkTemplate {
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
    pub errors: BookmarkFormErrors,
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub desc: String,
    pub category_id: Option<i32>,
    pub selected_tag_ids: Vec<i32>,
    pub new_tags: String,
    pub success_msg: Option<String>,
}

impl AddBookmarkTemplate {
    pub fn is_category_selected(&self, cat_id: &i32) -> bool {
        self.category_id.map_or(false, |id| id == *cat_id)
    }

    pub fn is_tag_selected(&self, tag_id: &i32) -> bool {
        self.selected_tag_ids.contains(tag_id)
    }
}

#[derive(Debug, Default)]
pub struct BookmarkFormErrors {
    pub title: Option<String>,
    pub url: Option<String>,
    pub cover_image: Option<String>,
    pub category_id: Option<String>,
    pub general: Option<String>,
}

#[derive(Debug)]
pub struct ValidateForm {
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub category_id: Option<i32>,
}

impl From<AddBookmarkForm> for ValidateForm {
    fn from(value: AddBookmarkForm) -> Self {
        ValidateForm {
            title: value.title,
            url: value.url,
            cover_image: value.cover_image,
            category_id: value.category_id,
        }
    }
}

impl From<EditBookmarkForm> for ValidateForm {
    fn from(value: EditBookmarkForm) -> Self {
        ValidateForm {
            title: value.title,
            url: value.url,
            cover_image: value.cover_image,
            category_id: value.category_id,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddBookmarkForm {
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub desc: String,
    pub category_id: Option<i32>,
    #[serde(default)]
    pub tag_ids: Vec<i32>,
    pub new_tags: String,
}

impl From<AddBookmarkForm> for BookmarkAdd {
    fn from(value: AddBookmarkForm) -> Self {
        let new_tags = value
            .new_tags
            .split(',')
            .filter_map(|s| match s.trim() {
                "" => None,
                s => Some(s.to_string()),
            })
            .collect::<Vec<String>>();

        BookmarkAdd {
            title: value.title,
            url: value.url,
            cover_image: value.cover_image,
            desc: value.desc,
            category_id: value.category_id.unwrap_or_default(),
            tags: value.tag_ids,
            new_tags,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteBookmarkForm {
    pub id: i32,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkImportForm {
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub category: String,
}

impl From<BookmarkImportForm> for BookmarkBatchAdd {
    fn from(value: BookmarkImportForm) -> Self {
        BookmarkBatchAdd {
            title: value.title,
            url: value.url,
            cover_image: value.cover_image,
            desc: String::new(),
            category: value.category,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ImportPayload {
    pub categories: Vec<String>,
    pub bookmarks: Vec<BookmarkImportForm>,
}

fn validate_bookmark_form(form: &ValidateForm) -> (BookmarkFormErrors, bool) {
    let mut errors = BookmarkFormErrors::default();

    if (&form.title.trim()).is_empty() {
        errors.title = Some("标题不能为空".into());
    }
    if (&form.url.trim()).is_empty() {
        errors.url = Some("URL不能为空".into());
    }
    if (&form.cover_image.trim()).is_empty() {
        errors.cover_image = Some("封面图片不能为空".into());
    }
    if form.category_id.is_none() {
        errors.category_id = Some("请选择分类".into());
    }

    let has_errors = errors.title.is_some()
        || errors.url.is_some()
        || errors.cover_image.is_some()
        || errors.category_id.is_some();

    (errors, has_errors)
}

pub async fn add_bookmark_form(
    State(service): State<BookmarkService<PgBookmarkRepository>>,
) -> impl IntoResponse {
    let (categories, tags) = service.get_categories_tags().await;
    let category_id = categories.first().map(|c| c.id);
    render_template(AddBookmarkTemplate {
        categories,
        tags,
        errors: BookmarkFormErrors::default(),
        title: String::new(),
        url: String::new(),
        cover_image: String::new(),
        desc: String::new(),
        category_id,
        selected_tag_ids: vec![],
        new_tags: String::new(),
        success_msg: None,
    })
}

pub async fn add_bookmark(
    State(service): State<BookmarkService<PgBookmarkRepository>>,
    Form(form): Form<AddBookmarkForm>,
) -> impl IntoResponse {
    let (categories, mut tags) = service.get_categories_tags().await;

    let (mut errors, has_errors) = validate_bookmark_form(&form.clone().into());

    if !has_errors {
        match service.add_bookmark(form.clone().into()).await {
            Ok(_) => {
                tags = service.get_tags().await.unwrap_or_default();
                let category_id = categories.first().map(|c| c.id);
                // 添加成功：重置表单
                return render_template(AddBookmarkTemplate {
                    categories,
                    tags,
                    errors: BookmarkFormErrors::default(),
                    title: String::new(),
                    url: String::new(),
                    cover_image: String::new(),
                    desc: String::new(),
                    category_id,
                    selected_tag_ids: vec![],
                    new_tags: String::new(),
                    success_msg: Some("添加成功".into()),
                });
            }
            Err(_) => errors.general = Some("添加失败，请稍后重试".into()),
        }
    }

    // 校验失败：保留用户输入
    render_template(AddBookmarkTemplate {
        categories,
        tags,
        errors,
        title: form.title,
        url: form.url,
        cover_image: form.cover_image,
        desc: form.desc,
        category_id: form.category_id,
        selected_tag_ids: form.tag_ids,
        new_tags: form.new_tags,
        success_msg: None,
    })
}

pub async fn delete_bookmark(
    State(service): State<BookmarkService<PgBookmarkRepository>>,
    Form(form): Form<DeleteBookmarkForm>,
) -> impl IntoResponse {
    let _ = service.delete_bookmark(form.id).await;
    Redirect::to("/")
}

// ─── Bookmark Edit ───────────────────────────────────────────────

#[derive(Debug, Template)]
#[template(path = "pages/edit_bookmark.html")]
pub struct EditBookmarkTemplate {
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
    pub errors: BookmarkFormErrors,
    pub id: i32,
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub desc: String,
    pub category_id: Option<i32>,
    pub selected_tag_ids: Vec<i32>,
    pub new_tags: String,
    pub success_msg: Option<String>,
}

impl EditBookmarkTemplate {
    pub fn is_category_selected(&self, cat_id: &i32) -> bool {
        self.category_id.map_or(false, |id| id == *cat_id)
    }

    pub fn is_tag_selected(&self, tag_id: &i32) -> bool {
        self.selected_tag_ids.contains(tag_id)
    }
}

#[derive(Debug, Deserialize)]
pub struct EditBookmarkQuery {
    pub id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EditBookmarkForm {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub desc: String,
    pub category_id: Option<i32>,
    #[serde(default)]
    pub tag_ids: Vec<i32>,
    pub new_tags: String,
}

impl From<EditBookmarkForm> for BookmarkUpdate {
    fn from(value: EditBookmarkForm) -> Self {
        let new_tags: Vec<String> = value
            .new_tags
            .split(',')
            .filter_map(|s| match s.trim() {
                "" => None,
                s => Some(s.to_string()),
            })
            .collect();

        BookmarkUpdate {
            id: value.id,
            title: value.title,
            url: value.url,
            cover_image: value.cover_image,
            desc: value.desc,
            category_id: value.category_id.unwrap_or_default(),
            tags: value.tag_ids,
            new_tags,
        }
    }
}

pub async fn get_edit_form(
    Query(q): Query<EditBookmarkQuery>,
    State(service): State<BookmarkService<PgBookmarkRepository>>,
) -> impl IntoResponse {
    let nav_detail = match service.get_bookmark_by_id(q.id).await {
        Ok(n) => n,
        Err(_) => return Redirect::to("/").into_response(),
    };
    let (categories, tags) = service.get_categories_tags().await;
    let selected_tag_ids = service.get_bookmark_tag_ids(q.id).await.unwrap_or_default();

    render_template(EditBookmarkTemplate {
        categories,
        tags,
        errors: BookmarkFormErrors::default(),
        id: nav_detail.id,
        title: nav_detail.title,
        url: nav_detail.url,
        cover_image: nav_detail.cover_image,
        desc: "".to_string(),
        category_id: Some(nav_detail.category_id),
        selected_tag_ids,
        new_tags: String::new(),
        success_msg: None,
    })
}

pub async fn edit_bookmark(
    State(service): State<BookmarkService<PgBookmarkRepository>>,
    Form(form): Form<EditBookmarkForm>,
) -> impl IntoResponse {
    let (mut errors, has_errors) = validate_bookmark_form(&form.clone().into());
    let (categories, tags) = service.get_categories_tags().await;
    if !has_errors {
        match service.update_bookmark(form.clone().into()).await {
            Ok(_) => return Redirect::to("/").into_response(),
            Err(_) => errors.general = Some("修改失败，请稍后重试".into()),
        }
    }

    render_template(EditBookmarkTemplate {
        categories,
        tags,
        errors,
        id: form.id,
        title: form.title,
        url: form.url,
        cover_image: form.cover_image,
        desc: form.desc,
        category_id: form.category_id,
        selected_tag_ids: form.tag_ids,
        new_tags: form.new_tags,
        success_msg: None,
    })
}

pub async fn bookmark_import_page() -> impl IntoResponse {
    match tokio::fs::read_to_string("static/bookmark-import.html").await {
        Ok(html) => Html(html).into_response(),
        Err(_) => e500("Failed to read bookmark-import.html").into_response(),
    }
}

pub async fn bookmark_import(
    State(service): State<BookmarkService<PgBookmarkRepository>>,
    Json(payload): Json<ImportPayload>,
) -> impl IntoResponse {
    let bookmarks: Vec<BookmarkBatchAdd> = payload.bookmarks.into_iter().map(Into::into).collect();
    let _ = service
        .batch_add_bookmarks(payload.categories, bookmarks)
        .await;
    Redirect::to("/")
}
