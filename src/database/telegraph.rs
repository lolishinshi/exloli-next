use sqlx::sqlite::SqliteQueryResult;
use sqlx::Result;

use super::db::DB;

#[derive(sqlx::FromRow, Debug)]
pub struct TelegraphEntity {
    /// 画廊 ID
    pub gallery_id: i32,
    /// telegraph 文章 URL
    pub url: String,
}

impl TelegraphEntity {
    pub async fn create(gallery_id: i32, telegraph: &str) -> Result<SqliteQueryResult> {
        sqlx::query("INSERT INTO telegraph (gallery_id, url) VALUES (?, ?)")
            .bind(gallery_id)
            .bind(telegraph)
            .execute(&*DB)
            .await
    }

    pub async fn get(gallery_id: i32) -> Result<Option<TelegraphEntity>> {
        sqlx::query_as("SELECT * FROM telegraph WHERE gallery_id = ?")
            .bind(gallery_id)
            .fetch_optional(&*DB)
            .await
    }

    pub async fn update(gallery_id: i32, telegraph: &str) -> Result<SqliteQueryResult> {
        sqlx::query("UPDATE telegraph SET telegraph = ? WHERE gallery_id = ?")
            .bind(telegraph)
            .bind(gallery_id)
            .execute(&*DB)
            .await
    }
}
