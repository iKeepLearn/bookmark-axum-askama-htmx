use crate::domain::bookmark::models::{Bookmark, CategoryWithBookmarkCount, QueryArgs, Tag};
use crate::domain::bookmark::services::BookmarkService;
use crate::infra::database::bookmark::PgBookmarkRepository;
use crate::utils::{e500, format_date_time, render_template};
use askama::Template;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use http::HeaderMap;
use serde::Deserialize;

use super::SessionUser;

#[derive(Debug)]
pub struct BookmarkItemTemplate {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

impl From<Bookmark> for BookmarkItemTemplate {
    fn from(item: Bookmark) -> Self {
        let tags: Vec<String> = item.tags.iter().map(|t| t.name.clone()).collect();
        BookmarkItemTemplate {
            id: item.id,
            title: item.title,
            url: item.url,
            cover_image: item.cover_image,
            tags,
            created_at: format_date_time(&item.created_at),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HomeQuery {
    #[serde(default)]
    pub c: Option<i32>,
    #[serde(default)]
    pub t: Option<String>,
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub page_size: Option<i32>,
}

#[derive(Debug, Template)]
#[template(path = "components/bookmark_list.html")]
pub struct BookmarkListTemplate {
    pub bookmarks: Vec<BookmarkItemTemplate>,
    pub next_page: i32,
    pub has_more: bool,
    pub selected_category: Option<i32>,
    pub selected_tag_ids: Vec<i32>,
    pub search_keyword: String,
    pub is_admin: bool,
}

impl BookmarkListTemplate {
    pub fn selected_tags_param(&self) -> String {
        self.selected_tag_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[derive(Debug, Template)]
#[template(path = "pages/home.html")]
pub struct HomeTemplate {
    pub bookmarks: BookmarkListTemplate,
    pub categories: Vec<CategoryWithBookmarkCount>,
    pub tags: Vec<Tag>,
    pub selected_category: Option<i32>,
    pub selected_tag_ids: Vec<i32>,
    pub search_keyword: String,
    pub user: SessionUser,
}

impl HomeTemplate {
    /// 生成点击某个分类后的 URL
    pub fn category_url(&self, cat_id: &i32) -> String {
        let mut params: Vec<String> = vec![];

        // 如果点击的是已选中的分类，则取消选中；否则选中该分类
        if self.selected_category.map_or(true, |c| c != *cat_id) {
            params.push(format!("c={}", cat_id));
        }

        // 保留已选标签
        if !self.selected_tag_ids.is_empty() {
            let tag_str: Vec<String> = self
                .selected_tag_ids
                .iter()
                .map(|id| id.to_string())
                .collect();
            params.push(format!("t={}", tag_str.join(",")));
        }

        // 保留搜索关键字
        if !self.search_keyword.is_empty() {
            params.push(format!("q={}", urlencoding::encode(&self.search_keyword)));
        }

        if params.is_empty() {
            "/".to_string()
        } else {
            format!("/?{}", params.join("&"))
        }
    }

    /// 生成点击某个标签后的 URL（切换该标签选中状态）
    pub fn tag_url(&self, tag_id: &i32) -> String {
        let mut params: Vec<String> = vec![];

        // 保留分类
        if let Some(cid) = self.selected_category {
            params.push(format!("c={}", cid));
        }

        // 切换标签
        let mut new_tags = self.selected_tag_ids.clone();
        if let Some(pos) = new_tags.iter().position(|id| id == tag_id) {
            new_tags.remove(pos);
        } else {
            new_tags.push(*tag_id);
        }

        if !new_tags.is_empty() {
            let tag_str: Vec<String> = new_tags.iter().map(|id| id.to_string()).collect();
            params.push(format!("t={}", tag_str.join(",")));
        }

        // 保留搜索关键字
        if !self.search_keyword.is_empty() {
            params.push(format!("q={}", urlencoding::encode(&self.search_keyword)));
        }

        if params.is_empty() {
            "/".to_string()
        } else {
            format!("/?{}", params.join("&"))
        }
    }

    pub fn is_category_selected(&self, cat_id: &i32) -> bool {
        self.selected_category.map_or(false, |c| c == *cat_id)
    }

    pub fn is_tag_selected(&self, tag_id: &i32) -> bool {
        self.selected_tag_ids.contains(tag_id)
    }

    pub fn is_all_selected(&self) -> bool {
        self.selected_category.is_none() && self.selected_tag_ids.is_empty()
    }

    pub fn total_bookmarks(&self) -> i64 {
        self.categories.iter().map(|cat| cat.bookmark_count).sum()
    }
}

#[axum::debug_handler]
pub async fn get_home(
    user: SessionUser,
    headers: HeaderMap,
    Query(query): Query<HomeQuery>,
    State(service): State<BookmarkService<PgBookmarkRepository>>,
) -> impl IntoResponse {
    let selected_tag_ids = parse_tag_ids(&query.t).unwrap_or_default();
    let search_keyword = query.q.unwrap_or_default();

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(100);
    let args = QueryArgs {
        category_id: query.c,
        tags: Some(selected_tag_ids.clone()),
        keyword: Some(search_keyword.clone()),
        page,
        page_size,
    };

    match service.get_bookmarks_by_query(&args).await {
        Ok(bookmarks) => {
            let categories = service
                .get_categories_with_bookmark_count()
                .await
                .unwrap_or_default();
            let tags = service.get_tags().await.unwrap_or_default();
            let has_more = bookmarks.len() as i32 == page_size;
            let bookmarks_template = BookmarkListTemplate {
                bookmarks: bookmarks.into_iter().map(Into::into).collect(),
                next_page: page + 1,
                has_more,
                selected_category: query.c,
                selected_tag_ids: selected_tag_ids.clone(),
                search_keyword: search_keyword.clone(),
                is_admin: user.is_admin,
            };
            if headers.get("HX-Request").is_some() {
                render_template(bookmarks_template)
            } else {
                render_template(HomeTemplate {
                    bookmarks: bookmarks_template,
                    categories,
                    tags,
                    selected_category: query.c,
                    selected_tag_ids,
                    search_keyword: search_keyword.clone(),
                    user,
                })
            }
        }
        Err(_) => e500("获取导航失败".to_string()),
    }
}

fn parse_tag_ids(t: &Option<String>) -> Option<Vec<i32>> {
    t.as_ref().map(|s| {
        s.split(',')
            .filter_map(|s| s.trim().parse::<i32>().ok())
            .collect()
    })
}
