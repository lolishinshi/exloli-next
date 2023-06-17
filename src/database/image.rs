use std::ops::Deref;

use sqlx::database::HasValueRef;
use sqlx::error::BoxDynError;
use sqlx::prelude::*;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Database, Result, Sqlite};

use super::db::DB;

#[derive(sqlx::FromRow, Debug)]
pub struct ImageEntity {
    /// 画廊 ID
    pub gallery_id: i32,
    /// 页面编号
    pub page: i32,
    /// 图片 hash
    pub hash: String,
    /// 相对 https://telegra.ph 的图片 URL
    pub url: String,
}

impl ImageEntity {
    /// 创建一条记录
    pub async fn create(
        gallery_id: i32,
        page: i32,
        hash: &str,
        url: &str,
    ) -> Result<SqliteQueryResult> {
        sqlx::query("INSERT INTO image (gallery_id, page, hash, url) VALUES (?, ?, ?, ?)")
            .bind(gallery_id)
            .bind(page)
            .bind(hash)
            .bind(url)
            .execute(&*DB)
            .await
    }

    /// 根据图片 hash 获取一张图片
    pub async fn get_by_hash(hash: &str) -> Result<Option<ImageEntity>> {
        sqlx::query_as("SELECT * FROM image WHERE hash = ?")
            .bind(hash)
            .fetch_optional(&*DB)
            .await
    }

    /// 从指定画廊里随机获取一张图片
    ///
    /// 画廊要求：
    /// - 分数大于 80
    /// - 作者只有 1 位
    pub async fn get_challenge() -> Result<Option<ImageEntity>> {
        sqlx::query_as(
            r#"SELECT * FROM image
            JOIN gallery ON gallery.id = image.gallery_id
            JOIN poll ON gallery.id = poll.gallery_id
            WHERE poll.score > 0.8
              AND JSON_ARRAY_LENGTH(JSON_EXTRACT(gallery.tags, '$.artist')) = 1
            ORDER BY RANDOM()
            LIMIT 1"#,
        )
        .fetch_optional(&*DB)
        .await
    }
}
