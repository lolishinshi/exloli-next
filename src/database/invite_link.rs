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
        let channel_id = CHANNEL_ID.get().unwrap();
        let now = Utc::now().naive_utc();
        sqlx::query!(
            "INSERT INTO invite_link (user_id, chat_id, link, created_at) VALUES (?, ?, ?, ?)",
            user_id,
            channel_id,
            link,
            now,
        )
        .execute(&*DB)
        .await
    }

    pub async fn get(user_id: i64) -> Result<Option<InviteLink>> {
        let channel_id = CHANNEL_ID.get().unwrap();
        sqlx::query_as!(InviteLink, "SELECT * FROM invite_link WHERE user_id = ? AND chat_id = ? ORDER BY created_at DESC LIMIT 1", user_id, channel_id)
            .fetch_optional(&*DB)
            .await
    }
}
