use chrono::{NaiveDate, Utc};
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Result;
use tracing::Level;

use super::db::DB;
use crate::config::CHANNEL_ID;

#[derive(sqlx::FromRow, Debug)]
pub struct MessageEntity {
    /// 消息 ID
    pub id: i32,
    /// 频道 ID
    pub channel_id: String,
    /// 画廊 ID
    pub gallery_id: i32,
    /// 消息发布日期
    pub publish_date: NaiveDate,
}

impl MessageEntity {
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn create(id: i32, gallery_id: i32) -> Result<SqliteQueryResult> {
        sqlx::query(
            "INSERT INTO message (id, channel_id, gallery_id, publish_date) VALUES (?, ?, ?, ?)",
        )
        .bind(id)
        .bind(CHANNEL_ID.get().unwrap())
        .bind(gallery_id)
        .bind(Utc::now().date_naive())
        .execute(&*DB)
        .await
    }

    // TODO: 如果存在与否不重要，其实不需要返回 Option，否则反而不方便上抛错误
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn get(id: i32) -> Result<Option<MessageEntity>> {
        sqlx::query_as("SELECT * FROM message WHERE id = ? AND channel_id = ?")
            .bind(id)
            .bind(CHANNEL_ID.get().unwrap())
            .fetch_optional(&*DB)
            .await
    }

    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn delete(id: i32) -> Result<SqliteQueryResult> {
        sqlx::query("DELETE FROM message WHERE id = ? AND channel_id = ?")
            .bind(id)
            .bind(CHANNEL_ID.get().unwrap())
            .execute(&*DB)
            .await
    }

    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn get_by_gallery_id(gallery_id: i32) -> Result<Option<MessageEntity>> {
        sqlx::query_as("SELECT * FROM message WHERE gallery_id = ? AND channel_id = ? ORDER BY publish_date DESC")
            .bind(gallery_id)
            .bind(CHANNEL_ID.get().unwrap())
            .fetch_optional(&*DB)
            .await
    }
}
