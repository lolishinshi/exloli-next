use chrono::prelude::*;
use sqlx::prelude::*;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Result;

use super::db::DB;

#[derive(FromRow)]
pub struct ChallengeView {
    pub id: i32,
    pub token: String,
    pub page: i32,
    pub artist: String,
    pub hash: String,
    pub url: String,
    pub score: f32,
}

pub struct ChallengeHistory {
    pub id: i32,
    pub user_id: i64,
    pub gallery_id: i32,
    pub page: i32,
    pub success: bool,
    pub answer_time: NaiveDateTime,
}

impl ChallengeView {
    pub async fn get_random() -> Result<Vec<Self>> {
        sqlx::query_as(
            r#"SELECT * FROM (
                SELECT * FROM (
                    SELECT * FROM challenge_view WHERE score > 0.8 ORDER BY random()
                ) GROUP BY artist
            ) ORDER BY random() LIMIT 4;"#,
        )
        .fetch_all(&*DB)
        .await
    }

    pub async fn verify(gallery_id: i32, artist: &str) -> Result<bool> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM challenge_view WHERE id = ? AND artist = ?)",
        )
        .bind(gallery_id)
        .bind(artist)
        .fetch_one(&*DB)
        .await
    }
}

impl ChallengeHistory {
    pub async fn create(
        user: i64,
        gallery: i32,
        page: i32,
        success: bool,
    ) -> Result<SqliteQueryResult> {
        sqlx::query(
            "INSERT INTO challenge_history (user_id, gallery_id, page, success, answer_time) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(user)
        .bind(gallery)
        .bind(page)
        .bind(success)
        .bind(Utc::now().naive_utc())
        .execute(&*DB)
        .await
    }
}
