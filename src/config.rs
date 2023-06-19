use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Deserialize;
use teloxide::types::{ChatId, Recipient};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// 日志等级
    pub log_level: String,
    /// 同时下载线程数量
    pub threads_num: usize,
    /// 定时爬取间隔，单位分钟
    pub interval: u64,
    /// Sqlite 数据库位置
    pub database_url: String,
    pub exhentai: ExHentai,
    pub telegraph: Telegraph,
    pub telegram: Telegram,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExHentai {
    /// 登陆 cookie
    pub cookie: String,
    /// 搜索参数
    pub search_params: Vec<(String, String)>,
    /// 最大搜索页面
    pub search_pages: usize,
    /// 过期天数，超过这个天数的本子不会进行更新 tag 等操作
    pub outdate: i64,
    /// 翻译文件的位置
    pub trans_file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Telegraph {
    /// Telegraph token
    pub access_token: String,
    /// 文章作者名称
    pub author_name: String,
    /// 文章作者连接
    pub author_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Telegram {
    /// 频道 id
    pub channel_id: Recipient,
    /// bot 名称
    pub bot_id: String,
    /// bot token
    pub token: String,
    /// 讨论组 ID
    pub group_id: ChatId,
}

impl Config {
    pub fn new(path: &str) -> Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&s)?)
    }
}
