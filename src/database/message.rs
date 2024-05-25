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
    pub async fn create(id: i32, gid: i32) -> Result<SqliteQueryResult> {
        let channel_id = CHANNEL_ID.get().unwrap();
        let now = Utc::now().date_naive();
        sqlx::query!(
            "INSERT INTO message (id, channel_id, gallery_id, publish_date) VALUES (?, ?, ?, ?)",
            id,
            channel_id,
            gid,
            now,
        )
        .execute(&*DB)
        .await
    }

    // TODO: 如果存在与否不重要，其实不需要返回 Option，否则反而不方便上抛错误
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn get(id: i32) -> Result<Option<MessageEntity>> {
        let channel_id = CHANNEL_ID.get().unwrap();
        sqlx::query_as!(
            MessageEntity,
            r#"
            SELECT
                id as "id: i32",
                channel_id,
                gallery_id as "gallery_id: i32",
                publish_date
            FROM message WHERE id = ? AND channel_id = ?
            "#,
            id,
            channel_id,
        )
        .fetch_optional(&*DB)
        .await
    }

    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn delete(id: i32) -> Result<SqliteQueryResult> {
        let channel_id = CHANNEL_ID.get().unwrap();
        sqlx::query!("DELETE FROM message WHERE id = ? AND channel_id = ?", id, channel_id)
            .execute(&*DB)
            .await
    }

    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn get_by_gallery(gid: i32) -> Result<Option<MessageEntity>> {
        let channel_id = CHANNEL_ID.get().unwrap();
        sqlx::query_as!(
            MessageEntity,
            r#"
            SELECT
                id as "id: i32",
                channel_id,
                gallery_id as "gallery_id: i32",
                publish_date
            FROM message
            WHERE gallery_id = ? AND channel_id = ?
            ORDER BY publish_date DESC
            "#,
            gid,
            channel_id
        )
        .fetch_optional(&*DB)
        .await
    }
}
