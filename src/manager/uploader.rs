use crate::config::Config;
use crate::ehentai::EHClient;
use anyhow::Result;
use futures::StreamExt;

#[derive(Debug)]
pub struct ExloliUploader {
    client: EHClient,
    config: Config,
}

impl ExloliUploader {
    pub async fn new(config: Config) -> Result<Self> {
        let client = EHClient::new(&config.exhentai.cookie).await?;
        Ok(Self { client, config })
    }

    pub async fn start(&self) {}

    /// 定时上传任务
    #[tracing::instrument(skip(self))]
    async fn task_upload(&self) -> Result<()> {
        let stream = self.client.search_iter(
            &self.config.exhentai.search_params,
            self.config.exhentai.search_pages,
        );
        tokio::pin!(stream);
        while let Some(next) = stream.next().await {}
        Ok(())
    }
}
