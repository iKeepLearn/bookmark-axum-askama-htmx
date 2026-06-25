use std::collections::HashMap;

use crate::domain::bookmark::error::BookmarkError;
use crate::domain::bookmark::models::{
    Bookmark as DBookmark, BookmarkAdd, BookmarkBatchAdd, BookmarkUpdate, Category as DCategory,
    CategoryWithBookmarkCount as Cwbm, QueryArgs, Tag as DTag,
};
use crate::domain::bookmark::traits::BookmarkRepository;
use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use tracing::error;

#[derive(Debug, FromRow, Clone)]
pub struct CategoryWithBookmarkCount {
    pub id: i32,
    pub name: String,
    pub bookmark_count: i64,
}

impl From<CategoryWithBookmarkCount> for Cwbm {
    fn from(value: CategoryWithBookmarkCount) -> Self {
        Cwbm {
            id: value.id,
            name: value.name,
            bookmark_count: value.bookmark_count,
        }
    }
}

#[derive(Debug, FromRow, Clone)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

impl From<Category> for DCategory {
    fn from(value: Category) -> Self {
        DCategory {
            id: value.id,
            name: value.name,
        }
    }
}

#[derive(Debug, FromRow, Clone, Deserialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

impl From<Tag> for DTag {
    fn from(value: Tag) -> Self {
        DTag {
            id: value.id,
            name: value.name,
        }
    }
}

#[derive(Debug, Clone, FromRow, Deserialize)]
pub struct Bookmark {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub cover_image: String,
    pub tags: Json<Vec<Tag>>,
    pub category_id: i32,
    pub created_at: DateTime<Utc>,
}

impl From<Bookmark> for DBookmark {
    fn from(value: Bookmark) -> Self {
        let tags: Vec<DTag> = value.tags.0.clone().into_iter().map(|t| t.into()).collect();
        DBookmark {
            id: value.id,
            title: value.title,
            url: value.url,
            cover_image: value.cover_image,
            tags,
            category_id: value.category_id,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PgBookmarkRepository {
    pub pool: PgPool,
}

impl PgBookmarkRepository {
    pub fn new(pool: PgPool) -> Self {
        PgBookmarkRepository { pool }
    }
}

impl BookmarkRepository for PgBookmarkRepository {
    async fn get_categories(&self) -> Result<Vec<DCategory>, BookmarkError> {
        match sqlx::query_as::<_, Category>(
            r#"
            SELECT id, name FROM categories ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(categories) => Ok(categories.into_iter().map(|c| c.into()).collect()),
            Err(e) => {
                error!("get_categories sqlx error {}", e);
                Err(BookmarkError::NotFound("categories".into()))
            }
        }
    }

    async fn get_tags(&self) -> Result<Vec<DTag>, BookmarkError> {
        match sqlx::query_as::<_, Tag>(
            r#"
            SELECT id, name FROM tags ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(tags) => Ok(tags.into_iter().map(|t| t.into()).collect()),
            Err(e) => {
                error!("get_tags sqlx error {}", e);
                Err(BookmarkError::NotFound("tags".into()))
            }
        }
    }

    async fn add_bookmark(&self, bookmark: BookmarkAdd) -> Result<(), BookmarkError> {
        let mut tx = self.pool.begin().await.context("begin transaction error")?;

        let bookmark_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO bookmarks (title, cover_image, url, "desc", category_id)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (url) DO UPDATE SET title = EXCLUDED.title, cover_image = EXCLUDED.cover_image, "desc" = EXCLUDED."desc", category_id = EXCLUDED.category_id
            RETURNING id
            "#,
        )
        .bind(bookmark.title)
        .bind(bookmark.cover_image)
        .bind(bookmark.url)
        .bind(bookmark.desc)
        .bind(bookmark.category_id)
        .fetch_one(&mut *tx)
        .await.context("add bookmark error")?;

        for tag_id in bookmark.tags {
            sqlx::query(
                r#"
                INSERT INTO bookmark_tags (bookmark_id, tag_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(bookmark_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await
            .context("add bookmark tag error")?;
        }

        for tag_name in &bookmark.new_tags {
            // 插入 tag（已存在则跳过）
            let tag_id: i32 = sqlx::query_scalar(
                r#"
                INSERT INTO tags (name)
                VALUES ($1)
                ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                RETURNING id
                "#,
            )
            .bind(tag_name)
            .fetch_one(&mut *tx)
            .await
            .context("add bookmark tag error")?;

            // 关联到 nav
            sqlx::query(
                r#"
                INSERT INTO bookmark_tags (bookmark_id, tag_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(bookmark_id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await
            .context("add bookmark tag error")?;
        }

        tx.commit().await.context("commit transaction error")?;
        Ok(())
    }
    async fn get_categories_with_bookmark_count(&self) -> Result<Vec<Cwbm>, BookmarkError> {
        match sqlx::query_as::<_, CategoryWithBookmarkCount>(
            r#"
            SELECT c.id, c.name, COUNT(b.id) AS bookmark_count
            FROM categories c
            LEFT JOIN bookmarks b ON c.id = b.category_id
            GROUP BY c.id, c.name
            ORDER BY c.id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(categories) => Ok(categories.into_iter().map(Into::into).collect()),
            Err(e) => {
                error!("get_categories_with_bookmark_count sqlx error {}", e);
                Err(BookmarkError::NotFound(
                    "categories_with_bookmark_count".into(),
                ))
            }
        }
    }

    async fn get_bookmarks_by_query(
        &self,
        query: &QueryArgs,
    ) -> Result<Vec<DBookmark>, BookmarkError> {
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 100);

        let offset = (page - 1) * page_size;
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                b.id,
                b.title,
                b.url,
                b.cover_image,
                b.desc,
                b.category_id,
                COALESCE(tags.tags, '[]') AS tags,
                b.created_at,
                b.updated_at
            FROM bookmarks b
            LEFT JOIN LATERAL (
                SELECT json_agg(
                    json_build_object('id', t.id, 'name', t.name)
                    ORDER BY t.id
                ) AS tags
                FROM bookmark_tags nt
                JOIN tags t ON t.id = nt.tag_id
                WHERE nt.bookmark_id = b.id
            ) tags ON true
            "#,
        );

        let mut has_where = false;

        // category filter
        if let Some(id) = query.category_id {
            qb.push(" WHERE b.category_id = ");
            qb.push_bind(id);
            has_where = true;
        }

        // keyword filter
        let keyword = query.keyword.as_deref().unwrap_or("").trim();
        if !keyword.is_empty() {
            if has_where {
                qb.push(" AND ");
            } else {
                qb.push(" WHERE ");
                has_where = true;
            }

            qb.push(" b.title ILIKE ");
            qb.push_bind(format!("%{}%", keyword));
        }

        if let Some(tag_ids) = &query.tags {
            if !tag_ids.is_empty() {
                if has_where {
                    qb.push(" AND ");
                } else {
                    qb.push(" WHERE ");
                }

                qb.push(
                    r#"
                EXISTS (
                    SELECT 1
                    FROM bookmark_tags nt2
                    WHERE nt2.bookmark_id = b.id
                    AND nt2.tag_id = ANY(
                "#,
                );

                qb.push_bind(tag_ids);

                qb.push("::int[]) )");
            }
        }

        qb.push(" ORDER BY b.created_at DESC, b.category_id ASC, b.title ASC");
        qb.push(" LIMIT ");
        qb.push_bind(page_size);

        qb.push(" OFFSET ");
        qb.push_bind(offset);
        let query = qb.build_query_as::<Bookmark>();

        match query.fetch_all(&self.pool).await {
            Ok(bookmarks) => Ok(bookmarks.into_iter().map(Into::into).collect()),
            Err(e) => {
                error!("get bookmarks by query error {}", e);
                Err(BookmarkError::NotFound("bookmarks".into()))
            }
        }
    }

    async fn delete_bookmark(&self, id: i32) -> Result<(), BookmarkError> {
        let query = r#"DELETE FROM bookmarks WHERE id = $1"#;
        match sqlx::query(query).bind(id).execute(&self.pool).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("delete bookmark error {}", e);
                Err(BookmarkError::NotFound("bookmark".into()))
            }
        }
    }

    async fn get_bookmark_by_id(&self, id: i32) -> Result<DBookmark, BookmarkError> {
        match sqlx::query_as::<_, Bookmark>(
            r#"
            SELECT
                b.id,
                b.title,
                b.url,
                b.cover_image,
                b.category_id,
                b.created_at,
                COALESCE(
                    json_agg(
                        json_build_object('id', t.id, 'name', t.name)
                        ORDER BY t.id
                    ) FILTER (WHERE t.id IS NOT NULL),
                    '[]'
                ) AS tags
            FROM bookmarks b
            LEFT JOIN bookmark_tags nt ON nt.bookmark_id = b.id
            LEFT JOIN tags t ON t.id = nt.tag_id
            WHERE b.id = $1
            GROUP BY b.id, b.title, b.url, b.cover_image, b.category_id, b.created_at
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        {
            Ok(bookmark) => Ok(bookmark.into()),
            Err(e) => {
                error!("get bookmark by id error {}", e);
                Err(BookmarkError::NotFound("bookmark".into()))
            }
        }
    }

    async fn update_bookmark(&self, bookmark: BookmarkUpdate) -> Result<(), BookmarkError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to begin update bookmark transaction")?;

        // 1. 更新 nav 基本字段
        sqlx::query(
            r#"
            UPDATE bookmarks
            SET title = $1, cover_image = $2, url = $3, "desc" = $4, category_id = $5, updated_at = NOW()
            WHERE id = $6
            "#,
        )
        .bind(bookmark.title)
        .bind(bookmark.cover_image)
        .bind(bookmark.url)
        .bind(bookmark.desc)
        .bind(bookmark.category_id)
        .bind(bookmark.id)
        .execute(&mut *tx)
        .await.context("failed to update bookmark")?;

        // 2. 删除旧标签关联
        sqlx::query("DELETE FROM bookmark_tags WHERE bookmark_id = $1")
            .bind(bookmark.id)
            .execute(&mut *tx)
            .await
            .context("failed to delete old tags")?;

        // 3. 关联已有标签
        for tag_id in bookmark.tags {
            sqlx::query(
                r#"
                INSERT INTO bookmark_tags (bookmark_id, tag_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(bookmark.id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await
            .context("failed to associate tags")?;
        }

        for tag_name in &bookmark.new_tags {
            let tag_id: i32 = sqlx::query_scalar(
                r#"
                INSERT INTO tags (name)
                VALUES ($1)
                ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                RETURNING id
                "#,
            )
            .bind(tag_name)
            .fetch_one(&mut *tx)
            .await
            .context("failed to insert new tag")?;

            sqlx::query(
                r#"
                INSERT INTO bookmark_tags (bookmark_id, tag_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(bookmark.id)
            .bind(tag_id)
            .execute(&mut *tx)
            .await
            .context("failed to associate new tag")?;
        }

        tx.commit()
            .await
            .context("failed to update bookmark transaction")?;
        Ok(())
    }

    async fn batch_add_bookmarks(
        &self,
        categories: Vec<String>,
        bookmarks: Vec<BookmarkBatchAdd>,
    ) -> Result<(), BookmarkError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to begin batch add bookmarks transaction")?;

        let mut category_map: HashMap<String, i32> = HashMap::new();

        if !categories.is_empty() {
            let rows: Vec<(i32, String)> = sqlx::query_as(
                r#"
                INSERT INTO categories (name)
                SELECT * FROM UNNEST($1::text[])
                ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
                RETURNING id, name
                "#,
            )
            .bind(&categories)
            .fetch_all(&mut *tx)
            .await
            .context("failed to insert categories")?;

            category_map.extend(rows.into_iter().map(|(id, name)| (name, id)));
        }

        if bookmarks.is_empty() {
            tx.commit()
                .await
                .context("failed to commit batch add categories transaction")?;
            return Ok(());
        }

        let mut dedup: HashMap<&str, &BookmarkBatchAdd> = HashMap::new();
        for b in &bookmarks {
            dedup.insert(b.url.as_str(), b);
        }

        let mut titles = Vec::with_capacity(dedup.len());
        let mut urls = Vec::with_capacity(dedup.len());
        let mut category_ids = Vec::with_capacity(dedup.len());
        let mut cover_images = Vec::with_capacity(dedup.len());

        for b in dedup.values() {
            let cid = category_map[&b.category];
            titles.push(b.title.as_str());
            urls.push(b.url.as_str());
            category_ids.push(cid);
            cover_images.push(b.cover_image.as_str());
        }

        sqlx::query(
            r#"
            INSERT INTO bookmarks (title, url, category_id, cover_image)
            SELECT * FROM UNNEST($1::text[], $2::text[], $3::int4[], $4::text[])
            ON CONFLICT (url) DO UPDATE
                SET title = EXCLUDED.title,
                    category_id = EXCLUDED.category_id,
                    cover_image = EXCLUDED.cover_image
            "#,
        )
        .bind(&titles)
        .bind(&urls)
        .bind(&category_ids)
        .bind(&cover_images)
        .execute(&mut *tx)
        .await
        .context("failed to insert bookmarks")?;

        tx.commit()
            .await
            .context("failed to commit batch add bookmarks transaction")?;
        Ok(())
    }

    async fn get_bookmark_tag_ids(&self, id: i32) -> Result<Vec<i32>, BookmarkError> {
        match sqlx::query_scalar::<_, i32>(
            r#"
            SELECT tag_id FROM bookmark_tags WHERE bookmark_id = $1 ORDER BY tag_id ASC
            "#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        {
            Ok(tags) => Ok(tags),
            Err(e) => {
                error!("get bookmark tag ids error {}", e);
                Err(BookmarkError::NotFound("bookmark tags".into()))
            }
        }
    }
}
