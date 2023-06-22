use std::env;

use anyhow::Result;
use exloli_next::bot::start_dispatcher;
use exloli_next::config::Config;
use exloli_next::ehentai::EhClient;
use exloli_next::uploader::ExloliUploader;
use exloli_next::utils::tags::EhTagTransDB;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new("./config.toml").unwrap();
    // NOTE: 全局数据库连接需要用这个变量初始化
    env::set_var("DATABASE_URL", &config.database_url);
    env::set_var("RUST_LOG", &config.log_level);

    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .unwrap();

    let ehentai = EhClient::new(&config.exhentai.cookie).await?;
    let bot = Bot::new(&config.telegram.token)
        .parse_mode(ParseMode::Html)
        .cache_me()
        .throttle(Default::default());
    let trans = EhTagTransDB::new(&config.exhentai.trans_file);

    let uploader =
        ExloliUploader::new(config.clone(), ehentai.clone(), bot.clone(), trans.clone()).await?;
    let uploader2 = uploader.clone();

    let t1 = tokio::spawn(async move { uploader.start().await });
    let t2 = tokio::spawn(async move { start_dispatcher(config, uploader2, bot, trans).await });

    tokio::try_join!(t1, t2)?;

    Ok(())
}
