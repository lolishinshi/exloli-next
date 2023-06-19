use anyhow::Result;
use exloli_next::bot::start_dispatcher;
use exloli_next::config::Config;
use exloli_next::ehentai::EhClient;
use exloli_next::manager::uploader::ExloliUploader;
use teloxide::Bot;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new("./config.toml").unwrap();
    let ehentai = EhClient::new(&config.exhentai.cookie).await?;
    let bot = Bot::new(&config.telegram.token);

    let uploader = ExloliUploader::new(config.clone(), ehentai.clone(), bot.clone()).await?;

    let t1 = tokio::spawn(async move { uploader.start().await });
    let t2 = tokio::spawn(async move { start_dispatcher(config, ehentai, bot).await });

    tokio::try_join!(t1, t2)?;

    Ok(())
}
