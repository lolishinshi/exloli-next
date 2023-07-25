use chrono::{NaiveDateTime, Utc};
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Result;

use super::db::DB;
use crate::config::CHANNEL_ID;

#[derive(sqlx::FromRow, Debug)]
pub struct InviteLink {
    pub user_id: i64,
    pub chat_id: String,
    pub link: String,
    pub created_at: NaiveDateTime,
}

impl InviteLink {
    pub async fn create(user_id: i64, link: &str) -> Result<SqliteQueryResult> {
        sqlx::query(
            "INSERT INTO invite_link (user_id, chat_id, link, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(CHANNEL_ID.get().unwrap())
        .bind(link)
        .bind(Utc::now().naive_utc())
        .execute(&*DB)
        .await
    }

    pub async fn get(user_id: i64) -> Result<Option<InviteLink>> {
        sqlx::query_as("SELECT * FROM invite_link WHERE user_id = ? AND chat_id = ? ORDER BY created_at DESC LIMIT 1")
            .bind(user_id)
            .bind(CHANNEL_ID.get().unwrap())
            .fetch_optional(&*DB)
            .await
    }
}
