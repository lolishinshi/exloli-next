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
    pub chat_id: i64,
}

impl ChallengeView {
    pub async fn get_random() -> Result<Vec<Self>> {
        sqlx::query_as(
            r#"
            SELECT * FROM (
                -- 此处使用 group by 嵌套 random，因为默认情况下 group by 只会显示每组的第一个结果
                SELECT * FROM (
                    SELECT * FROM challenge_view
                    WHERE score > 0.8 AND id NOT IN (
                        -- 此处过滤掉出现在大于 5 个画廊中的图片，因为大概率是广告
                        -- 还有第一页和最后一页
                        SELECT image_id FROM page GROUP BY image_id HAVING COUNT(gallery_id) > 5
                        UNION
                        SELECT image_id FROM page GROUP BY gallery_id HAVING page = MAX(page)
                        UNION
                        SELECT image_id FROM page GROUP BY gallery_id HAVING page = 1
                    ) ORDER BY random()
                ) GROUP BY artist
            ) ORDER BY random() LIMIT 4"#,
        )
        .fetch_all(&*DB)
        .await
    }
}

impl ChallengeHistory {
    pub async fn create(
        user: i64,
        gallery: i32,
        page: i32,
        success: bool,
        chat_id: i64,
    ) -> Result<SqliteQueryResult> {
        sqlx::query(
            "INSERT INTO challenge_history (user_id, gallery_id, page, success, answer_time, chat_id) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(user)
        .bind(gallery)
        .bind(page)
        .bind(success)
        .bind(Utc::now().naive_utc())
        .bind(chat_id)
        .execute(&*DB)
        .await
    }
}
