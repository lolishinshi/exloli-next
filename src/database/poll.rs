use chrono::{NaiveDateTime, Utc};
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Row;

use super::db::DB;

#[derive(sqlx::FromRow, Debug)]
pub struct Poll {
    /// 投票 ID
    pub id: i64,
    /// 画廊 ID
    pub gallery_id: i32,
    /// 当前投票的分数，为 0~1 的小数
    pub score: f32,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Vote {
    /// 用户 ID
    pub user_id: i32,
    /// 投票 ID
    pub poll_id: i64,
    /// 投票选项
    pub option: i32,
    /// 投票时间
    pub vote_time: NaiveDateTime,
}

impl Poll {
    pub async fn create(id: i64, gallery_id: i32) -> sqlx::Result<SqliteQueryResult> {
        sqlx::query("INSERT INTO poll (id, gallery_id, score) VALUES (?, ?, 0.0)")
            .bind(id)
            .bind(gallery_id)
            .execute(&*DB)
            .await
    }

    pub async fn get_by_gallery_id(gallery_id: i32) -> sqlx::Result<SqliteQueryResult> {
        sqlx::query("SELECT * FROM poll WHERE gallery_id = ?")
            .bind(gallery_id)
            .execute(&*DB)
            .await
    }

    pub async fn get_vote(id: i64) -> sqlx::Result<[i32; 5]> {
        let mut result = [0; 5];
        let rows = sqlx::query(
            "SELECT option, COUNT(option) FROM poll JOIN vote ON poll.id = vote.poll_id WHERE poll.id = ? GROUP BY option"
        )
            .bind(id)
            .fetch_all(&*DB)
            .await?;
        for row in rows {
            result[row.get::<u32, _>(0) as usize - 1] = row.get(1);
        }
        Ok(result)
    }

    async fn update_score(id: i64) -> sqlx::Result<f32> {
        let vote = Self::get_vote(id).await?;
        let score = wilson_score(&vote);
        sqlx::query("UPDATE poll SET score = ? WHERE id = ?")
            .bind(score)
            .bind(id)
            .execute(&*DB)
            .await?;
        Ok(score)
    }
}

impl Vote {
    pub async fn create(
        user_id: i32,
        poll_id: i64,
        option: i32,
    ) -> sqlx::Result<SqliteQueryResult> {
        let result = sqlx::query(
            "INSERT INTO vote (user_id, poll_id, option, vote_time) VALUES (?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(poll_id)
        .bind(option)
        .bind(Utc::now().naive_utc())
        .execute(&*DB)
        .await?;
        Poll::update_score(poll_id).await?;
        Ok(result)
    }
}

/// 威尔逊得分
/// 基于：https://www.jianshu.com/p/4d2b45918958
pub fn wilson_score(votes: &[i32]) -> f32 {
    let base = [0., 0.25, 0.5, 0.75, 1.];
    let count = votes.iter().sum::<i32>() as f32;
    if count == 0. {
        return 0.;
    }
    let mean = Iterator::zip(votes.iter(), base.iter())
        .map(|(&a, &b)| a as f32 * b)
        .sum::<f32>()
        / count;
    let var = Iterator::zip(votes.iter(), base.iter())
        .map(|(&a, &b)| (mean - b).powi(2) * a as f32)
        .sum::<f32>()
        / count;
    // 80% 置信度
    let z = 1.281f32;

    (mean + z.powi(2) / (2. * count) - ((z / (2. * count)) * (4. * count * var + z.powi(2)).sqrt()))
        / (1. + z.powi(2) / count)
}
