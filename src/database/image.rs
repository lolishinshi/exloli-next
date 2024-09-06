use sqlx::sqlite::SqliteQueryResult;
use sqlx::Result;
use tracing::Level;

use super::db::DB;

#[derive(sqlx::FromRow, Debug)]
pub struct PageEntity {
    /// 画廊 ID
    pub gallery_id: i32,
    /// 页面编号
    pub page: i32,
    /// 图片 id
    pub image_id: u32,
}

#[derive(sqlx::FromRow, Debug)]
pub struct ImageEntity {
    /// 图片在 E 站的 fileindex
    pub id: u32,
    /// 图片的 sha1sum 前 10 位
    pub hash: String,
    /// 相对 https://telegra.ph 的图片 URL
    url: String,
}

impl ImageEntity {
    /// 创建一条记录
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn create(id: u32, hash: &str, url: &str) -> Result<SqliteQueryResult> {
        sqlx::query!("INSERT OR IGNORE INTO image (id, hash, url) VALUES (?, ?, ?)", id, hash, url)
            .execute(&*DB)
            .await
    }

    /// 根据图片 hash 获取一张图片
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn get_by_hash(hash: &str) -> Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id as "id: u32", hash, url FROM image WHERE hash = ?"#,
            hash
        )
        .fetch_optional(&*DB)
        .await
    }

    /// 获取指定画廊的所有图片，并且按页码排列
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn get_by_gallery_id(gallery_id: i32) -> Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT
                image.id as "id: u32",
                image.hash as hash,
                image.url as url
            FROM image
            JOIN page ON page.image_id = image.id
            WHERE page.gallery_id = ?
            ORDER BY page.page
            "#,
            gallery_id,
        )
        .fetch_all(&*DB)
        .await
    }

    pub fn url(&self) -> String {
        if self.url.starts_with("/file/") {
            format!("https://telegra.ph{}", self.url)
        } else {
            self.url.clone()
        }
    }
}

impl PageEntity {
    /// 创建一条记录，有冲突时则忽略
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn create(gallery_id: i32, page: i32, image_id: u32) -> Result<SqliteQueryResult> {
        sqlx::query!(
            "INSERT OR IGNORE INTO page (gallery_id, page, image_id) VALUES (?, ?, ?)",
            gallery_id,
            page,
            image_id
        )
        .execute(&*DB)
        .await
    }

    /// 统计某个画廊的有记录页面数量
    pub async fn count(gallery_id: i32) -> Result<i32> {
        sqlx::query_scalar!("SELECT COUNT(*) FROM page WHERE gallery_id = ?", gallery_id)
            .fetch_one(&*DB)
            .await
    }
}
