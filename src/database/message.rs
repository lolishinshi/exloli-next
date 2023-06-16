use chrono::{NaiveDate, NaiveDateTime, Utc};
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Result, SqlitePool};

use super::db::DB;

#[derive(sqlx::FromRow, Debug)]
pub struct Message {
    /// 消息 ID
    pub id: i32,
    /// 画廊 ID
    pub gallery_id: i32,
    /// telegraph 文章 URL
    pub telegraph: String,
    /// 消息发布日期
    pub publish_date: NaiveDate,
}

impl Message {
    pub async fn create(
        id: i32,
        gallery_id: i32,
        telegraph: &str,
        upload_images: i32,
    ) -> Result<SqliteQueryResult> {
        sqlx::query(
            "REPLACE INTO publish (id, gallery_id, telegraph, upload_images, publish_date) VALUES (?, ?, ?, ?, ?)"
        )
            .bind(id)
            .bind(gallery_id)
            .bind(telegraph)
            .bind(upload_images)
            .bind(Utc::now().date_naive())
            .execute(&*DB)
            .await
    }

    pub async fn get(id: i32) -> Result<Option<Message>> {
        sqlx::query_as("SELECT * FROM publish WHERE id = ?")
            .bind(id)
            .fetch_optional(&*DB)
            .await
    }

    pub async fn update_telegraph(id: i32, telegraph: &str) -> Result<SqliteQueryResult> {
        sqlx::query("UPDATE publish SET telegraph = ? WHERE id = ?")
            .bind(telegraph)
            .bind(id)
            .execute(&*DB)
            .await
    }
}
