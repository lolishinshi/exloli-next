use std::env;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use exloli_next::config::Config;
use exloli_next::ehentai::EhClient;
use futures::StreamExt;
use glob::glob;
use tracing::{info, warn};

#[derive(Parser)]
struct Args {
    /// 配置文件路径
    #[clap(short, long, default_value = "./config.toml")]
    config: String,
    /// 收藏分类
    #[clap(short, long, default_value = "0")]
    favcat: u32,
    /// H@H 下载位置
    #[clap(short, long, default_value = "/mnt/ehentai/download/convert")]
    download: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = Config::new(&args.config)?;

    env::set_var("RUST_LOG", &config.log_level);

    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .unwrap();

    let ehentai = EhClient::new(&config.exhentai.cookie).await?;
    let params = [("favcat", args.favcat)];
    let stream = ehentai.page_iter("https://exhentai.org/favorites.php", &params);
    tokio::pin!(stream);
    while let Some(gallery) = stream.next().await {
        if glob(&format!("{}/*[[]{}]", args.download, gallery.id()))?.next().is_some() {
            info!("跳过: {}", gallery.url());
            continue;
        }
        info!("请求下载: {}", gallery.url());
        if let Err(err) = ehentai.archive_gallery(&gallery).await {
            warn!("下载失败: {}", err);
        } else {
            tokio::time::sleep(Duration::from_secs(1800)).await;
        }
    }
    Ok(())
}
