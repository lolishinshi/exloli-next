use std::ops::Deref;

use chrono::NaiveDate;
use indexmap::IndexMap;
use sqlx::database::HasValueRef;
use sqlx::error::BoxDynError;
use sqlx::prelude::*;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Database, Result, Sqlite};
use tracing::Level;

use super::db::DB;

// 此处使用 IndexMap，因为我们需要保证相同的 tag 每次序列化的结果都是一样的
#[derive(Debug, Default)]
pub struct TagsEntity(pub IndexMap<String, Vec<String>>);

#[derive(Debug, FromRow)]
pub struct GalleryEntity {
    /// 画廊 ID
    pub id: i32,
    /// 画廊 token
    pub token: String,
    /// 画廊标题
    pub title: String,
    /// 画廊日文标题
    pub title_jp: Option<String>,
    /// JSON 格式的画廊标签
    /// 旧画廊可能为空
    pub tags: TagsEntity,
    /// 页面数量
    pub pages: i32,
    /// 父画廊
    pub parent: Option<i32>,
    /// 是否已删除
    pub deleted: bool,
}

impl GalleryEntity {
    /// 创建一条记录
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn create(
        id: i32,
        token: &str,
        title: &str,
        title_jp: &Option<String>,
        tags: &IndexMap<String, Vec<String>>,
        pages: i32,
        parent: Option<i32>,
    ) -> Result<SqliteQueryResult> {
        sqlx::query("REPLACE INTO gallery (id, token, title, title_jp, tags, pages, parent, deleted) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(id)
            .bind(token)
            .bind(title)
            .bind(title_jp)
            .bind(serde_json::to_string(tags).unwrap())
            .bind(pages)
            .bind(parent)
            .bind(false)
            .execute(&*DB)
            .await
    }

    /// 根据 ID 获取一条记录
    ///
    /// 注意，此处不会返回已被标记为删除的记录
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn get(id: i32) -> Result<Option<GalleryEntity>> {
        sqlx::query_as("SELECT * FROM gallery WHERE id = ? AND deleted = FALSE")
            .bind(id)
            .fetch_optional(&*DB)
            .await
    }

    /// 根据消息 ID 获取一条记录
    pub async fn get_by_msg(id: i32) -> Result<Option<GalleryEntity>> {
        sqlx::query_as(
            "SELECT * FROM gallery JOIN message ON gallery.id = message.gallery_id WHERE message.id = ? AND gallery.deleted = FALSE"
        )
            .bind(id)
            .fetch_optional(&*DB)
            .await
    }

    /// 检查画廊是否存在，此处不会考虑删除标记
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn check(id: i32) -> Result<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM gallery WHERE id = ?)")
            .bind(id)
            .fetch_one(&*DB)
            .await
    }

    /// 根据 ID 更新 tag
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn update_tags(id: i32, tags: &[(String, Vec<String>)]) -> Result<SqliteQueryResult> {
        sqlx::query("UPDATE gallery SET tags = ? WHERE id = ?")
            .bind(serde_json::to_string(tags).unwrap())
            .bind(id)
            .execute(&*DB)
            .await
    }

    /// 根据 ID 更新删除状态
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn update_deleted(id: i32, deleted: bool) -> Result<SqliteQueryResult> {
        sqlx::query("UPDATE gallery SET deleted = ? WHERE id = ?")
            .bind(deleted)
            .bind(id)
            .execute(&*DB)
            .await
    }

    /// 彻底删除一个画廊
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn delete(id: i32) -> Result<SqliteQueryResult> {
        sqlx::query("DELETE FROM gallery WHERE id = ?").bind(id).execute(&*DB).await
    }

    /// 查询自指定日期以来的本子，结果按分数从高到低排列
    /// 返回 分数、标题、消息 ID
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn list(
        start: NaiveDate,
        end: NaiveDate,
        limit: i32,
        page: i32,
    ) -> Result<Vec<(f32, String, i32)>> {
        sqlx::query_as(
            r#"SELECT poll.score, gallery.title, message.id
            FROM gallery
            JOIN poll ON poll.gallery_id = gallery.id
            JOIN message ON message.gallery_id = gallery.id
            WHERE message.publish_date BETWEEN ? AND ?
            ORDER BY poll.score DESC LIMIT ? OFFSET ?"#,
        )
        .bind(start)
        .bind(end)
        .bind(limit)
        .bind(page * limit)
        .fetch_all(&*DB)
        .await
    }
}

impl<'q> Decode<'q, Sqlite> for TagsEntity {
    fn decode(
        value: <Sqlite as HasValueRef<'q>>::ValueRef,
    ) -> std::result::Result<Self, BoxDynError> {
        let str = <String as Decode<Sqlite>>::decode(value)?;
        if str.is_empty() {
            Ok(TagsEntity(IndexMap::new()))
        } else {
            Ok(TagsEntity(serde_json::from_str(&str)?))
        }
    }
}

impl Type<Sqlite> for TagsEntity {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <String as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &<Sqlite as Database>::TypeInfo) -> bool {
        <String as Type<Sqlite>>::compatible(ty)
    }
}

impl Deref for TagsEntity {
    type Target = IndexMap<String, Vec<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn gallery() {
        let mut tags = IndexMap::new();
        tags.insert("female".to_owned(), vec!["dress".to_owned(), "maid".to_owned()]);
        GalleryEntity::create(123, "token", "test gallery", &None, &tags, 0, None).await.unwrap();
        let g = GalleryEntity::get(123).await.unwrap().unwrap();
        assert_eq!(g.tags.0, tags);
        assert!(GalleryEntity::check(123).await.unwrap());
    }
}
