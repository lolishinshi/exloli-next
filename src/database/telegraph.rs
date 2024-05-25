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
        sqlx::query!(
            "REPLACE INTO telegraph (gallery_id, url) VALUES (?, ?)",
            gallery_id,
            telegraph
        )
        .execute(&*DB)
        .await
    }

    pub async fn get(gallery_id: i32) -> Result<Option<TelegraphEntity>> {
        sqlx::query_as!(
            TelegraphEntity,
            r#"SELECT gallery_id as "gallery_id: i32", url FROM telegraph WHERE gallery_id = ?"#,
            gallery_id
        )
        .fetch_optional(&*DB)
        .await
    }

    pub async fn update(gallery_id: i32, telegraph: &str) -> Result<SqliteQueryResult> {
        sqlx::query!("UPDATE telegraph SET url = ? WHERE gallery_id = ?", telegraph, gallery_id)
            .execute(&*DB)
            .await
    }
}
