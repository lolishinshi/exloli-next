use sqlx::sqlite::SqliteQueryResult;
use sqlx::{FromRow, Result};

use super::db::DB;

#[derive(FromRow)]
pub struct TagsEntity(pub Vec<(String, Vec<String>)>);

#[derive(FromRow)]
pub struct GalleryEntity {
    /// 画廊 ID
    pub id: i32,
    /// 画廊 token
    pub token: String,
    /// 画廊标题
    pub title: String,
    /// JSON 格式的画廊标签
    #[sqlx(try_from = "String")]
    pub tags: TagsEntity,
    /// 父画廊
    pub parent: Option<i32>,
}

impl GalleryEntity {
    pub async fn create(
        id: i32,
        token: &str,
        title: &str,
        tags: &TagsEntity,
        parent: Option<i32>,
    ) -> Result<SqliteQueryResult> {
        sqlx::query("INSERT INTO gallery (id, token, title, tags, parent) VALUES (?, ?, ?, ?, ?)")
            .bind(id)
            .bind(token)
            .bind(title)
            .bind(serde_json::to_string(&tags.0).unwrap())
            .bind(parent)
            .execute(&*DB)
            .await
    }

    pub async fn get(id: i32) -> Result<Option<GalleryEntity>> {
        sqlx::query_as("SELECT * FROM gallery WHERE id = ?")
            .bind(id)
            .fetch_optional(&*DB)
            .await
    }

    pub async fn update_tags(id: i32, tags: TagsEntity) -> Result<SqliteQueryResult> {
        sqlx::query("UPDATE gallery SET tags = ? WHERE id = ?")
            .bind(serde_json::to_string(&tags.0).unwrap())
            .bind(id)
            .execute(&*DB)
            .await
    }
}

impl TryFrom<String> for TagsEntity {
    type Error = serde_json::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Ok(TagsEntity(serde_json::from_str(&value)?))
    }
}
