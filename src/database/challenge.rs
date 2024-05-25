use chrono::prelude::*;
use sqlx::prelude::*;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Result;

use super::db::DB;

#[derive(FromRow, Clone)]
pub struct ChallengeView {
    pub id: i32,
    pub token: String,
    pub page: i32,
    pub artist: String,
    pub image_id: i32,
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
        sqlx::query_as!(
            Self,
            r#"
            SELECT
                id as "id: i32",
                token,
                page as "page: i32",
                artist as "artist!",
                image_id as "image_id: i32",
                url,
                score as "score: f32"
            FROM (
                -- 此处使用 group by 嵌套 random，因为默认情况下 group by 只会显示每组的第一个结果
                SELECT * FROM (
                    SELECT * FROM challenge_view
                    WHERE score > 0.8 AND image_id NOT IN (
                        -- 此处过滤掉出现在大于 5 个画廊中的图片，因为大概率是广告
                        -- 还有第一页和最后一页
                        -- 这个查询太耗时了，现在有基于二维码的过滤了，暂时禁用看一下效果
                        -- SELECT image_id FROM page GROUP BY image_id HAVING COUNT(gallery_id) > 5
                        -- UNION
                        SELECT image_id FROM page GROUP BY gallery_id HAVING page = MAX(page)
                        UNION
                        SELECT image_id FROM page GROUP BY gallery_id HAVING page = 1
                    ) ORDER BY random() LIMIT 500 -- 限制结果数量来提高速度，500 个结果一般能凑齐 4 个作者了
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
        let now = Utc::now().naive_utc();
        sqlx::query!(
            "INSERT INTO challenge_history (user_id, gallery_id, page, success, answer_time, chat_id) VALUES (?, ?, ?, ?, ?, ?)",
            user,
            gallery,
            page,
            success,
            now,
            chat_id,
        )
        .execute(&*DB)
        .await
    }

    pub async fn answer_stats(user: i64, chat_id: i64) -> Result<(i32, i32)> {
        let record = sqlx::query!(
            r#"SELECT SUM(success) as "success!", COUNT(*) as "total!" FROM challenge_history WHERE user_id = ? AND chat_id = ?"#, user, chat_id,
        )
        .fetch_one(&*DB)
        .await?;
        Ok((record.success, record.total))
    }
}
