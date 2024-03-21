use chrono::{NaiveDateTime, Utc};
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Result, Row};
use tracing::Level;

use super::db::DB;

#[derive(sqlx::FromRow, Debug)]
pub struct PollEntity {
    /// 投票 ID
    /// NOTE: 仅部分早期投票 id 范围是 i64
    pub id: i64,
    /// 画廊 ID
    pub gallery_id: i32,
    /// 当前投票的分数，为 0~1 的小数
    pub score: f32,
    /// 旧系统的投票数据，代表 1~5 的投票数量
    pub old_vote: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct VoteEntity {
    /// 用户 ID
    pub user_id: i64,
    /// 投票 ID
    pub poll_id: i64,
    /// 投票选项
    pub option: i32,
    /// 投票时间
    pub vote_time: NaiveDateTime,
}

impl PollEntity {
    /// 插入一条记录，如果冲突则忽略
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn create(id: i64, gallery_id: i32) -> Result<SqliteQueryResult> {
        sqlx::query("INSERT OR IGNORE INTO poll (id, gallery_id, score) VALUES (?, ?, 0.0)")
            .bind(id)
            .bind(gallery_id)
            .execute(&*DB)
            .await
    }

    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn get_by_gallery(gallery_id: i32) -> Result<Option<Self>> {
        sqlx::query_as("SELECT * FROM poll WHERE gallery_id = ?")
            .bind(gallery_id)
            .fetch_optional(&*DB)
            .await
    }

    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn get_vote(id: i64) -> Result<[i32; 5]> {
        let mut result = [0; 5];
        let rows = sqlx::query(
            r#"
            SELECT option, SUM(count) FROM (
                SELECT option, COUNT(option) AS count FROM poll JOIN vote ON poll.id = vote.poll_id WHERE poll.id = ? GROUP BY option
                UNION ALL
                SELECT key + 1 AS option, value AS count FROM poll, json_each(poll.old_vote) WHERE poll.id = ?
            ) GROUP BY option
            "#
        )
            .bind(id)
            .bind(id)
            .fetch_all(&*DB)
            .await?;
        for row in rows {
            result[row.get::<u32, _>(0) as usize - 1] = row.get(1);
        }
        Ok(result)
    }

    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn update_score(id: i64) -> Result<f32> {
        let vote = Self::get_vote(id).await?;
        let score = wilson_score(&vote);
        sqlx::query("UPDATE poll SET score = ? WHERE id = ?")
            .bind(score)
            .bind(id)
            .execute(&*DB)
            .await?;
        Ok(score)
    }

    /// 获取指定投票的分数排名区段，结果为一个 0~1 的小数
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn rank(&self) -> Result<f32> {
        let (higher, total): (i32, i32) = sqlx::query_as(
            "SELECT COUNT(*), (SELECT COUNT(*) FROM poll) FROM poll WHERE score > ?",
        )
        .bind(self.score)
        .fetch_one(&*DB)
        .await?;
        Ok(higher as f32 / total as f32)
    }
}

impl VoteEntity {
    /// 创建一个用户投票，创建完毕后请调用 PollEntity::update_score 来更新分数
    #[tracing::instrument(level = Level::DEBUG)]
    pub async fn create(user_id: u64, poll_id: i64, option: i32) -> Result<SqliteQueryResult> {
        sqlx::query("REPLACE INTO vote (user_id, poll_id, option, vote_time) VALUES (?, ?, ?, ?)")
            .bind(user_id as i64)
            .bind(poll_id)
            .bind(option)
            .bind(Utc::now().naive_utc())
            .execute(&*DB)
            .await
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
    let mean =
        Iterator::zip(votes.iter(), base.iter()).map(|(&a, &b)| a as f32 * b).sum::<f32>() / count;
    let var = Iterator::zip(votes.iter(), base.iter())
        .map(|(&a, &b)| (mean - b).powi(2) * a as f32)
        .sum::<f32>()
        / count;
    // 80% 置信度
    let z = 1.281f32;

    (mean + z.powi(2) / (2. * count) - ((z / (2. * count)) * (4. * count * var + z.powi(2)).sqrt()))
        / (1. + z.powi(2) / count)
}
