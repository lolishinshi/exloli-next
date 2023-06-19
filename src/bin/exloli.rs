use anyhow::Result;
use exloli_next::config::Config;
use exloli_next::ehentai::EhClient;
use exloli_next::manager::uploader::ExloliUploader;
use teloxide::Bot;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new("./config.toml").unwrap();
    let ehclient = EhClient::new(&config.exhentai.cookie).await?;
    let bot = Bot::new(&config.telegram.token);

    let uploader = ExloliUploader::new(config, ehclient, bot).await?;

    tokio::spawn(uploader.start());

    Ok(())
}
