use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Deserialize;
use teloxide::types::{ChatId, Recipient};

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let path = std::env::var("CONFIG_FILE").unwrap_or_else(|_| "config.toml".to_string());
    Config::new(&path).unwrap()
});

#[derive(Debug, Deserialize)]
pub struct Config {
    pub log_level: String,
    pub threads_num: usize,
    pub interval: u64,
    pub database_url: String,
    pub exhentai: ExHentai,
    pub telegraph: Telegraph,
    pub telegram: Telegram,
}

#[derive(Debug, Deserialize)]
pub struct ExHentai {
    pub cookie: Option<String>,
    pub search_params: Vec<(String, String)>,
    pub search_pages: i32,
    pub outdate: i64,
}

#[derive(Debug, Deserialize)]
pub struct Telegraph {
    pub access_token: String,
    pub author_name: String,
    pub author_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Telegram {
    pub channel_id: Recipient,
    pub bot_id: String,
    pub token: String,
    pub group_id: ChatId,
}

impl Config {
    pub fn new(path: &str) -> Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&s)?)
    }
}
