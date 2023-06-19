use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::format::parse;
use futures::{future, stream, StreamExt, TryStreamExt};
use reqwest::Client;
use telegraph_rs::{html_to_node, Error, Telegraph};
use teloxide::Bot;
use tokio::sync::Semaphore;
use tracing::{debug, error};

use crate::config::Config;
use crate::database::{GalleryEntity, ImageEntity, PageEntity};
use crate::ehentai::{EhClient, EhGallery, EhGalleryUrl, EhPageUrl};
use crate::utils::imagebytes::ImageBytes;
use crate::utils::pad_left;
use crate::utils::tags::EhTagTransDB;

#[derive(Debug, Clone)]
pub struct ExloliUploader {
    ehentai: EhClient,
    telegraph: Telegraph,
    bot: Bot,
    config: Config,
    trans: Arc<EhTagTransDB>,
}

impl ExloliUploader {
    pub async fn new(config: Config, ehentai: EhClient, bot: Bot) -> Result<Self> {
        let telegraph = Telegraph::new(&config.telegraph.author_name)
            .author_url(&config.telegraph.author_url)
            .access_token(&config.telegraph.access_token)
            .create()
            .await?;
        let trans = Arc::new(EhTagTransDB::new(&config.exhentai.trans_file));
        Ok(Self {
            ehentai,
            config,
            telegraph,
            bot,
            trans,
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
            self.check_and_update(&next).await?;
            self.check_and_upload(&next).await?;
        }
        Ok(())
    }

    /// 检查指定画廊是否已经上传，如果没有则进行上传
    ///
    /// 为了避免绕晕自己，这次不考虑父子画廊，只要 id 不同就视为新画廊，只要是新画廊就进行上传
    #[tracing::instrument(skip(self))]
    async fn check_and_upload(&self, gallery: &EhGalleryUrl) -> Result<()> {
        if GalleryEntity::get(gallery.id()).await?.is_some() {
            return Ok(());
        }

        let gallery = self.ehentai.get_gallery(&gallery).await?;
        self.upload_gallery_image(&gallery).await?;
        let article = self.publish_telegraph_article(&gallery).await?;

        todo!()
    }

    /// 检查指定画廊是否有更新，比如标题、标签
    #[tracing::instrument(skip(self))]
    async fn check_and_update(&self, gallery: &EhGalleryUrl) -> Result<()> {
        todo!()
    }
}

impl ExloliUploader {
    /// 获取某个画廊里的所有图片，并且上传到 telegrpah，如果已经上传过的，会跳过上传
    async fn upload_gallery_image(&self, gallery: &EhGallery) -> Result<()> {
        // 扫描所有图片
        // 对于已经上传过的图片，不需要重复上传，只需要插入 PageEntity 记录即可
        let mut pages = vec![];
        for page in &gallery.pages {
            match ImageEntity::get_by_hash(page.hash()).await? {
                Some(img) => {
                    PageEntity::create(page.gallery_id(), page.page(), img.id).await?;
                }
                None => pages.push(page.clone()),
            }
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let client = self.ehentai.clone();
        let concurrent = self.config.threads_num;

        // E 站的图片是分布式存储的，所以并行下载
        let downloader = tokio::spawn(async move {
            let mut stream = stream::iter(pages)
                .map(|page| {
                    let client = client.clone();
                    async move { (page.clone(), client.get_image_bytes(&page).await) }
                })
                .buffered(concurrent);

            while let Some((page, bytes)) = stream.next().await {
                debug!("finish download image: {:?}", page);
                match bytes {
                    Ok(bytes) => tx.send((page, bytes))?,
                    Err(e) => error!("download {:?} error: {:?}", page, e),
                }
            }

            Result::<()>::Ok(())
        });

        // 依次将图片上传到 telegraph，并插入 ImageEntity 和 PageEntity 记录
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
        let uploader = tokio::spawn(async move {
            // TODO: 此处可以考虑一次上传多个图片，减少请求次数，避免触发 telegraph 的 rate limit
            while let Some((page, bytes)) = rx.recv().await {
                let resp = Telegraph::upload_with(&[ImageBytes(bytes)], &client).await?;
                let image = ImageEntity::create(page.hash(), &resp[0].src).await?;
                PageEntity::create(page.gallery_id(), page.page(), image.id).await?;
            }
            Result::<()>::Ok(())
        });

        let (first, second) = tokio::try_join!(downloader, uploader)?;
        first?;
        second?;

        Ok(())
    }

    /// 从数据库中读取某个画廊的所有图片，生成一篇 telegraph 文章
    async fn publish_telegraph_article(&self, gallery: &EhGallery) -> Result<telegraph_rs::Page> {
        let images = ImageEntity::get_by_gallery_id(gallery.url.id()).await?;

        let mut html = String::new();
        for img in images {
            html.push_str(&format!(r#"<img src="{}">"#, img.url));
        }
        html.push_str(&format!("<p>图片总数：{}</p>", gallery.pages.len()));

        let node = html_to_node(&html);
        Ok(self
            .telegraph
            .create_page(&gallery.title, &node, false)
            .await?)
    }

    /// 从数据库中读取某个画廊的信息，生成一条可供发送的 telegram 消息正文
    async fn create_gallery_message_text(&self, gallery: &EhGallery) -> Result<String> {
        let gallery = GalleryEntity::get(gallery.url.id()).await?.unwrap();

        // 首先，将 tag 翻译
        // 并整理成 namespace: #tag1 #tag2 #tag3 的格式
        let tags = self.trans.trans_tags(&gallery.tags.0);
        let mut text = String::new();
        for (ns, tag) in tags {
            let tag = tag
                .iter()
                .map(|s| format!("#{}", s))
                .collect::<Vec<_>>()
                .join(" ");
            text.push_str(&format!("<code>{}</code>: {}\n", pad_left(&ns, 6), tag))
        }

        todo!()
    }
}
