use std::ops::Deref;

use indexmap::IndexMap;
use sqlx::database::HasValueRef;
use sqlx::error::BoxDynError;
use sqlx::prelude::*;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Database, Result, Sqlite};

use super::db::DB;

// 此处使用 IndexMap，因为我们需要保证相同的 tag 每次序列化的结果都是一样的
#[derive(Debug)]
pub struct TagsEntity(pub IndexMap<String, Vec<String>>);

#[derive(Debug, FromRow)]
pub struct GalleryEntity {
    /// 画廊 ID
    pub id: i32,
    /// 画廊 token
    pub token: String,
    /// 画廊标题
    pub title: String,
    /// JSON 格式的画廊标签
    pub tags: TagsEntity,
    /// 页面数量
    pub pages: i32,
    /// 父画廊
    pub parent: Option<i32>,
    /// 是否已删除
    pub deleted: bool,
}

impl GalleryEntity {
    /// 创建一条记录
    pub async fn create(
        id: i32,
        token: &str,
        title: &str,
        tags: &[(String, Vec<String>)],
        pages: i32,
        parent: Option<i32>,
    ) -> Result<SqliteQueryResult> {
        sqlx::query("INSERT INTO gallery (id, token, title, tags, pages, parent, deleted) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(id)
            .bind(token)
            .bind(title)
            .bind(serde_json::to_string(tags).unwrap())
            .bind(pages)
            .bind(parent)
            .bind(false)
            .execute(&*DB)
            .await
    }

    /// 根据 ID 获取一条记录
    pub async fn get(id: i32) -> Result<Option<GalleryEntity>> {
        sqlx::query_as("SELECT * FROM gallery WHERE id = ?")
            .bind(id)
            .fetch_optional(&*DB)
            .await
    }

    /// 根据 ID 更新 tag
    pub async fn update_tags(id: i32, tags: &[(String, Vec<String>)]) -> Result<SqliteQueryResult> {
        sqlx::query("UPDATE gallery SET tags = ? WHERE id = ?")
            .bind(serde_json::to_string(tags).unwrap())
            .bind(id)
            .execute(&*DB)
            .await
    }
}

impl<'q> Decode<'q, Sqlite> for TagsEntity {
    fn decode(
        value: <Sqlite as HasValueRef<'q>>::ValueRef,
    ) -> std::result::Result<Self, BoxDynError> {
        let str = <String as Decode<Sqlite>>::decode(value)?;
        Ok(TagsEntity(serde_json::from_str(&str)?))
    }
}

impl Type<Sqlite> for TagsEntity {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <String as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &<Sqlite as Database>::TypeInfo) -> bool {
        <String as Type<Sqlite>>::compatible(ty)
    }
}

impl Deref for TagsEntity {
    type Target = IndexMap<String, Vec<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
