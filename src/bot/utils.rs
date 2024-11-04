use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use dashmap::DashMap;
use image::EncodableLayout;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::database::ChallengeView;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallbackData {
    /// 投票、选项
    VoteForPoll(i64, i32),
    /// 开始、结束、偏移
    NextPage(i32, i32, i32),
    /// 开始、结束、偏移
    PrevPage(i32, i32, i32),
    /// 挑战 ID、画师名称
    Challenge(i64, String),
}

impl CallbackData {
    pub fn pack(&self) -> String {
        match self {
            Self::VoteForPoll(a, b) => format!("vote {} {}", a, b),
            Self::NextPage(a, b, c) => format!("> {} {} {}", a, b, c),
            Self::PrevPage(a, b, c) => format!("< {} {} {}", a, b, c),
            Self::Challenge(a, b) => format!("challenge {}:{}", a, b),
        }
    }

    pub fn unpack(s: &str) -> Option<Self> {
        let (cmd, data) = s.split_once(' ')?;
        match cmd {
            "vote" => {
                let (a, b) = data.split_once(' ')?;
                Some(Self::VoteForPoll(a.parse().ok()?, b.parse().ok()?))
            }
            ">" => {
                let (a, data) = data.split_once(' ')?;
                let (b, c) = data.split_once(' ')?;
                Some(Self::NextPage(a.parse().ok()?, b.parse().ok()?, c.parse().ok()?))
            }
            "<" => {
                let (a, data) = data.split_once(' ')?;
                let (b, c) = data.split_once(' ')?;
                Some(Self::PrevPage(a.parse().ok()?, b.parse().ok()?, c.parse().ok()?))
            }
            "challenge" => {
                let (a, b) = data.split_once(':')?;
                Some(Self::Challenge(a.parse().ok()?, b.to_string()))
            }
            _ => None,
        }
    }
}

/// 一个用于限制请求频率的数据结构
#[derive(Debug, Clone)]
pub struct RateLimiter(Arc<RateLimiterInner>);

#[derive(Debug)]
struct RateLimiterInner {
    interval: std::time::Duration,
    limit: usize,
    data: DashMap<UserId, VecDeque<Instant>>,
}

impl RateLimiter {
    pub fn new(interval: std::time::Duration, limit: usize) -> Self {
        assert_ne!(limit, 0);
        Self(Arc::new(RateLimiterInner { interval, limit, data: Default::default() }))
    }

    /// 插入数据，正常情况下返回 None，如果达到了限制则返回需要等待的时间
    pub fn insert(&self, key: UserId) -> Option<std::time::Duration> {
        let mut entry = self.0.data.entry(key).or_default();
        let entry = entry.value_mut();
        // 插入时，先去掉已经过期的元素
        while let Some(first) = entry.front() {
            if first.elapsed() > self.0.interval {
                entry.pop_front();
            } else {
                break;
            }
        }
        if entry.len() == self.0.limit {
            return entry.front().cloned().map(|d| self.0.interval - d.elapsed());
        }
        entry.push_back(Instant::now());
        None
    }
}

/// 防止快速点击导致重复答题
#[derive(Debug, Clone)]
pub struct ChallengeLocker(Arc<DashMap<i64, (i32, i32, String)>>);

impl ChallengeLocker {
    pub fn new() -> Self {
        Self(Arc::new(Default::default()))
    }

    /// 添加一个挑战，返回这个挑战的随机 ID
    pub fn add_challenge(&self, gallery: i32, page: i32, artist: String) -> i64 {
        let key = rand::random::<i64>();
        self.0.insert(key, (gallery, page, artist));
        key
    }

    /// 尝试获得一个答题机会
    pub fn get_challenge(&self, id: i64) -> Option<(i32, i32, String)> {
        Some(self.0.remove(&id)?.1)
    }
}

/// 时刻缓存一些有效的挑战，提高响应速度
#[derive(Debug, Clone)]
pub struct ChallengeProvider(Arc<Mutex<Receiver<Vec<ChallengeView>>>>);

impl ChallengeProvider {
    pub fn new() -> Self {
        let (tx, rx) = channel(5);
        tokio::spawn(async move {
            loop {
                match Self::_get_challenge().await {
                    Ok(challenge) => {
                        tx.send(challenge).await.unwrap();
                    }
                    Err(e) => {
                        warn!("获取挑战失败: {}", e);
                    }
                }
            }
        });
        Self(Arc::new(Mutex::new(rx)))
    }

    async fn _get_challenge() -> Result<Vec<ChallengeView>> {
        loop {
            let challenge = ChallengeView::get_random().await?;
            if challenge.is_empty() {
                sleep(Duration::from_secs(5)).await;
                continue;
            }
            let answer = &challenge[0];
            let url = if answer.url.starts_with("https://") {
                answer.url.clone()
            } else {
                format!("https://telegra.ph{}", answer.url)
            };
            let resp = reqwest::get(&url).await?;
            let data = resp.bytes().await?;
            if has_qrcode(&data)? {
                info!("跳过包含二维码的图片");
                continue;
            }
            return Ok(challenge);
        }
    }

    pub async fn get_challenge(&self) -> Option<Vec<ChallengeView>> {
        self.0.lock().await.recv().await
    }
}

pub fn has_qrcode(data: &[u8]) -> Result<bool> {
    let image = image::load_from_memory(data)?.into_luma8();
    let mut decoder = quircs::Quirc::default();
    let codes = decoder.identify(image.width() as usize, image.height() as usize, image.as_bytes());
    Ok(codes.count() > 0)
}
