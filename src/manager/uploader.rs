use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::format::parse;
use futures::{future, stream, StreamExt, TryStreamExt};
use telegraph_rs::Telegraph;
use tokio::sync::Semaphore;
use tracing::{debug, error};

use crate::config::Config;
use crate::database::{GalleryEntity, ImageEntity};
use crate::ehentai::{EhClient, EhGallery, EhGalleryUrl};

#[derive(Debug)]
pub struct ExloliUploader {
    ehentai: EhClient,
    telegraph: Telegraph,
    config: Config,
}

impl ExloliUploader {
    pub async fn new(config: Config) -> Result<Self> {
        let ehentai = EhClient::new(&config.exhentai.cookie).await?;
        let telegraph = Telegraph::new(&config.telegraph.author_name)
            .author_url(&config.telegraph.author_url)
            .access_token(&config.telegraph.access_token)
            .create()
            .await?;
        Ok(Self {
            ehentai,
            config,
            telegraph,
        })
    }

    /// 每隔 interval 分钟检查一次
    pub async fn start(&self) {
        loop {
            if let Err(e) = self.check().await {
                error!("task loop error: {:?}", e);
            }
            tokio::time::sleep(Duration::from_secs(self.config.interval * 60)).await;
        }
    }

    /// 根据配置文件，扫描前 N 页的本子，并进行上传或者更新
    #[tracing::instrument(skip(self))]
    async fn check(&self) -> Result<()> {
        let stream = self.ehentai.search_iter(
            &self.config.exhentai.search_params,
            self.config.exhentai.search_pages,
        );
        tokio::pin!(stream);
        while let Some(next) = stream.next().await {
            self.check_update(&next).await?;
            self.check_upload(&next).await?;
        }
        Ok(())
    }

    /// 检查指定画廊是否已经上传，如果没有则进行上传
    ///
    /// 为了避免绕晕自己，这次不考虑父子画廊，只要 id 不同就视为新画廊，只要是新画廊就进行上传
    #[tracing::instrument(skip(self))]
    async fn check_upload(&self, gallery: &EhGalleryUrl) -> Result<()> {
        if GalleryEntity::get(gallery.id()).await?.is_some() {
            return Ok(());
        }

        let gallery = self.ehentai.get_gallery(gallery.url()).await?;
        for page in &gallery.pages {
            let image = self.ehentai.get_image_url(&page).await?;
        }

        todo!()
    }

    /// 检查指定画廊是否有更新，比如标题、标签
    #[tracing::instrument(skip(self))]
    async fn check_update(&self, gallery: &EhGalleryUrl) -> Result<()> {
        todo!()
    }
}

/// 获取某个画廊里的所有图片，并且上传到 telegrpah
/// 返回上传后的链接
///
/// 这个函数会缓存上传结果
async fn upload_gallery_image(client: EhClient, gallery: EhGallery, concurrent: usize) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // 考虑到 E 站的图片是分布式存储的，所以可以并行下载，但是上传只能串行
    let downloader = tokio::spawn(async move {
        let mut stream = stream::iter(gallery.pages)
            // 过滤掉已经上传过的图片
            .filter(|page| {
                let page = page.clone();
                async move {
                    ImageEntity::get_by_hash(page.hash())
                        .await
                        .map(|r| r.is_some())
                        .unwrap_or_default()
                }
            })
            .map(|page| {
                let client = client.clone();
                async move { (page.clone(), client.get_image_bytes(&page).await) }
            })
            .buffered(concurrent);

        tokio::pin!(stream);

        while let Some((page, bytes)) = stream.next().await {
            debug!("finish download image: {:?}", page);
            match bytes {
                Ok(bytes) => tx.send((page, bytes))?,
                Err(e) => error!("download {:?} error: {:?}", page, e),
            }
        }

        Result::<()>::Ok(())
    });

    let uploader = tokio::spawn(async move { while let Some((page, bytes)) = rx.recv().await {} });

    tokio::join!(downloader, uploader);
}
