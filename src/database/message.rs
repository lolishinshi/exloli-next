use chrono::{NaiveDate, Utc};
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Result;
use tracing::Level;

use super::db::DB;

#[derive(sqlx::FromRow, Debug)]
pub struct MessageEntity {
    /// 消息 ID
    pub id: i32,
    /// 画廊 ID
    pub gallery_id: i32,
    /// telegraph 文章 URL
    pub telegraph: String,
    /// 消息发布日期
    pub publish_date: NaiveDate,
}

impl MessageEntity {
    #[tracing::instrument(level = Level::TRACE)]
    pub async fn create(id: i32, gallery_id: i32, telegraph: &str) -> Result<SqliteQueryResult> {
        sqlx::query(
            "REPLACE INTO publish (id, gallery_id, telegraph, publish_date) VALUES (?, ?, ?, ?)",
        )
        .bind(id)
        .bind(gallery_id)
        .bind(telegraph)
        .bind(Utc::now().date_naive())
        .execute(&*DB)
        .await
    }

    #[tracing::instrument(level = Level::TRACE)]
    pub async fn get_by_gallery_id(gallery_id: i32) -> Result<Option<MessageEntity>> {
        sqlx::query_as("SELECT * FROM publish WHERE gallery_id = ?")
            .bind(gallery_id)
            .fetch_optional(&*DB)
            .await
    }

    #[tracing::instrument(level = Level::TRACE)]
    pub async fn update_telegraph(id: i32, telegraph: &str) -> Result<SqliteQueryResult> {
        sqlx::query("UPDATE publish SET telegraph = ? WHERE id = ?")
            .bind(telegraph)
            .bind(id)
            .execute(&*DB)
            .await
    }
}
