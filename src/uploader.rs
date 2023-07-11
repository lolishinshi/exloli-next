use std::backtrace::Backtrace;
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::{Datelike, Utc};
use futures::StreamExt;
use regex::Regex;
use reqwest::{Client, StatusCode};
use telegraph_rs::{html_to_node, Telegraph};
use teloxide::prelude::*;
use teloxide::types::MessageId;
use teloxide::utils::html::{code_inline, link};
use tokio::time;
use tracing::{debug, error, info, instrument, Instrument};

use crate::bot::Bot;
use crate::config::Config;
use crate::database::{GalleryEntity, ImageEntity, MessageEntity, PageEntity};
use crate::ehentai::{EhClient, EhGallery, EhGalleryUrl, GalleryInfo};
use crate::tags::EhTagTransDB;
use crate::utils::imagebytes::ImageBytes;
use crate::utils::pad_left;

#[derive(Debug, Clone)]
pub struct ExloliUploader {
    ehentai: EhClient,
    telegraph: Telegraph,
    bot: Bot,
    config: Config,
    trans: EhTagTransDB,
}

impl ExloliUploader {
    pub async fn new(
        config: Config,
        ehentai: EhClient,
        bot: Bot,
        trans: EhTagTransDB,
    ) -> Result<Self> {
        let telegraph = Telegraph::new(&config.telegraph.author_name)
            .author_url(&config.telegraph.author_url)
            .access_token(&config.telegraph.access_token)
            .create()
            .await?;
        Ok(Self { ehentai, config, telegraph, bot, trans })
    }

    /// 每隔 interval 分钟检查一次
    pub async fn start(&self) {
        loop {
            info!("开始扫描 E 站 本子");
            self.check().await;
            info!("扫描完毕，等待 {:?} 后继续", self.config.interval);
            time::sleep(self.config.interval).await;
        }
    }

    /// 根据配置文件，扫描前 N 个本子，并进行上传或者更新
    #[tracing::instrument(skip(self))]
    async fn check(&self) {
        let stream = self
            .ehentai
            .search_iter(&self.config.exhentai.search_params)
            .take(self.config.exhentai.search_count);
        tokio::pin!(stream);
        while let Some(next) = stream.next().await {
            // 错误不要上抛，避免影响后续画廊
            if let Err(err) = self.try_update(&next, true).await {
                error!("check_and_update: {:?}\n{}", err, Backtrace::force_capture());
            }
            if let Err(err) = self.try_upload(&next, true).await {
                error!("check_and_upload: {:?}\n{}", err, Backtrace::force_capture());
            }
            time::sleep(Duration::from_secs(1)).await;
        }
    }

    /// 检查指定画廊是否已经上传，如果没有则进行上传
    ///
    /// 为了避免绕晕自己，这次不考虑父子画廊，只要 id 不同就视为新画廊，只要是新画廊就进行上传
    #[tracing::instrument(skip(self))]
    pub async fn try_upload(&self, gallery: &EhGalleryUrl, check: bool) -> Result<()> {
        if check && GalleryEntity::check(gallery.id()).await? {
            return Ok(());
        }

        let gallery = self.ehentai.get_gallery(gallery).await?;
        // 上传图片、发布文章
        self.upload_gallery_image(&gallery).await?;
        let article = self.publish_telegraph_article(&gallery).await?;
        // 发送消息
        let text = self.create_message_text(&gallery, &article.url).await?;
        // FIXME: 此处没有考虑到父画廊没有上传，但是父父画廊上传过的情况
        // 不过一般情况下画廊应该不会那么短时间内更新多次
        let msg = if let Some(parent) = &gallery.parent {
            if let Some(pmsg) = MessageEntity::get_by_gallery_id(parent.id()).await? {
                self.bot
                    .send_message(self.config.telegram.channel_id.clone(), text)
                    .reply_to_message_id(MessageId(pmsg.id))
                    .await?
            } else {
                self.bot.send_message(self.config.telegram.channel_id.clone(), text).await?
            }
        } else {
            self.bot.send_message(self.config.telegram.channel_id.clone(), text).await?
        };
        // 数据入库
        MessageEntity::create(msg.id.0, gallery.url.id(), &article.url).await?;
        GalleryEntity::create(&gallery).await?;

        Ok(())
    }

    /// 检查指定画廊是否有更新，比如标题、标签
    #[tracing::instrument(skip(self))]
    pub async fn try_update(&self, gallery: &EhGalleryUrl, check: bool) -> Result<()> {
        let entity = match GalleryEntity::get(gallery.id()).await? {
            Some(v) => v,
            _ => return Ok(()),
        };
        let message = MessageEntity::get_by_gallery_id(gallery.id()).await?.unwrap();

        // 2 天内创建的画廊，每天都尝试更新
        // 7 天内创建的画廊，每 3 天尝试更新
        // 14 天内创建的画廊，每 7 天尝试更新
        // 其余的，每 14 天尝试更新
        let now = Utc::now().date_naive();
        let seed = match now - message.publish_date {
            d if d < chrono::Duration::days(2) => 1,
            d if d < chrono::Duration::days(7) => 3,
            d if d < chrono::Duration::days(14) => 7,
            _ => 14,
        };
        if check && now.day() % seed != 0 {
            return Ok(());
        }

        // 检查 tag 和标题是否有变化
        let gallery = self.ehentai.get_gallery(gallery).await?;

        if gallery.tags != entity.tags.0 || gallery.title != entity.title {
            let text = self.create_message_text(&gallery, &message.telegraph).await?;
            self.bot
                .edit_message_text(
                    self.config.telegram.channel_id.clone(),
                    MessageId(message.id),
                    text,
                )
                .await?;
        }

        GalleryEntity::create(&gallery).await?;

        Ok(())
    }

    /// 重新发布指定画廊的文章，并更新消息
    pub async fn republish(&self, gallery: &GalleryEntity, msg: &MessageEntity) -> Result<()> {
        info!("重新发布：{} {}", gallery.url(), msg.id);
        let article = self.publish_telegraph_article(gallery).await?;
        let text = self.create_message_text(gallery, &article.url).await?;
        self.bot
            .edit_message_text(self.config.telegram.channel_id.clone(), MessageId(msg.id), text)
            .await?;
        MessageEntity::update_telegraph(gallery.id, &article.url).await?;
        Ok(())
    }

    /// 检查 telegraph 文章是否正常
    pub async fn check_telegraph(&self, url: &str) -> Result<bool> {
        Ok(Client::new().head(url).send().await?.status() != StatusCode::NOT_FOUND)
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
                    // NOTE: 此处存在重复插入的可能，但是由于 PageEntity::create 使用 OR IGNORE，所以不影响
                    PageEntity::create(page.gallery_id(), page.page(), img.id).await?;
                }
                None => pages.push(page.clone()),
            }
        }
        info!("需要下载&上传 {} 张图片", pages.len());

        let concurrent = self.config.threads_num;
        let (tx, mut rx) = tokio::sync::mpsc::channel(concurrent * 2);
        let client = self.ehentai.clone();

        // 获取图片链接时不要并行，避免触发反爬限制
        let getter = tokio::spawn(
            async move {
                for page in pages {
                    let rst = client.get_image_url(&page).await?;
                    info!("已解析：{}", page.page());
                    tx.send((page, rst)).await?;
                }
                Result::<()>::Ok(())
            }
            .in_current_span(),
        );

        // TODO: 实现并行
        // 依次将图片下载并上传到 telegraph，并插入 ImageEntity 和 PageEntity 记录
        // TODO: telegraph 上传应该也不用并行，只有图片下载因为时分布式存储，并行也没关系
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(30))
            .build()?;
        let uploader = tokio::spawn(
            async move {
                // TODO: 此处可以考虑一次上传多个图片，减少请求次数，避免触发 telegraph 的 rate limit
                while let Some((page, (fileindex, url))) = rx.recv().await {
                    let bytes = client.get(url).send().await?.bytes().await?.to_vec();
                    debug!("已下载: {}", page.page());
                    let resp = Telegraph::upload_with(&[ImageBytes(bytes)], &client).await?;
                    debug!("已上传: {}", page.page());
                    ImageEntity::create(fileindex, page.hash(), &resp[0].src).await?;
                    PageEntity::create(page.gallery_id(), page.page(), fileindex).await?;
                }
                Result::<()>::Ok(())
            }
            .in_current_span(),
        );

        let (first, second) = tokio::try_join!(getter, uploader)?;
        first?;
        second?;

        Ok(())
    }

    /// 从数据库中读取某个画廊的所有图片，生成一篇 telegraph 文章
    /// 为了防止画廊被删除后无法更新，此处不应该依赖 EhGallery
    async fn publish_telegraph_article<T: GalleryInfo>(
        &self,
        gallery: &T,
    ) -> Result<telegraph_rs::Page> {
        let images = ImageEntity::get_by_gallery_id(gallery.url().id()).await?;

        let mut html = String::new();
        for img in images {
            html.push_str(&format!(r#"<img src="{}">"#, img.url));
        }
        html.push_str(&format!("<p>图片总数：{}</p>", gallery.pages()));

        let node = html_to_node(&html);
        // 文章标题优先使用日文
        let title = gallery.title_jp();
        Ok(self.telegraph.create_page(&title, &node, false).await?)
    }

    /// 为画廊生成一条可供发送的 telegram 消息正文
    async fn create_message_text<T: GalleryInfo>(
        &self,
        gallery: &T,
        article: &str,
    ) -> Result<String> {
        // 首先，将 tag 翻译
        // 并整理成 namespace: #tag1 #tag2 #tag3 的格式
        let re = Regex::new("[-/· ]").unwrap();
        let tags = self.trans.trans_tags(gallery.tags());
        let mut text = String::new();
        for (ns, tag) in tags {
            let tag = tag
                .iter()
                .map(|s| format!("#{}", re.replace_all(s, "_")))
                .collect::<Vec<_>>()
                .join(" ");
            text.push_str(&format!("{}: {}\n", code_inline(&pad_left(&ns, 6)), tag))
        }

        text.push_str(
            &format!("{}: {}\n", code_inline("  预览"), link(article, &gallery.title()),),
        );
        text.push_str(&format!("{}: {}", code_inline("原始地址"), gallery.url().url()));

        Ok(text)
    }
}

impl ExloliUploader {
    /// 重新扫描并更新所有历史画廊，
    // TODO: 该功能需要移除
    pub async fn rescan(&self, start: usize, end: usize) -> Result<()> {
        let galleries = GalleryEntity::all().await?;
        for gallery in galleries.iter().skip(start).take(end - start) {
            info!("更新画廊 {}", gallery.url());
            if let Err(err) = self.rescan_gallery(gallery).await {
                error!("更新失败 {}", err);
            }
        }
        Ok(())
    }

    #[instrument(skip_all, fields(gallery = %gallery.url()))]
    pub async fn rescan_gallery(&self, gallery: &GalleryEntity) -> Result<()> {
        // 重新扫描缺页或者根本没有记录页面的本子
        if gallery.posted.is_none()
            || gallery.pages == 0
            || gallery.pages == 200  // TODO: 因为某个旧 BUG，某些画廊的页数只爬到了 200 页，处理完这些画廊后可以去掉这行
            || gallery.pages != PageEntity::count(gallery.id).await?
        {
            let gallery = self.ehentai.get_gallery(&gallery.url()).await?;
            self.reupload_gallery(&gallery).await?;
            GalleryEntity::create(&gallery).await?;
        }
        let msg =
            MessageEntity::get_by_gallery_id(gallery.id).await?.ok_or(anyhow!("找不到消息"))?;
        if !self.check_telegraph(&msg.telegraph).await? {
            self.republish(gallery, &msg).await?;
        }
        time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    /// 获取某个画廊里的所有图片，并添加记录
    // TODO: 该功能需要移除或者合并到 upload_gallery_image 中
    async fn reupload_gallery(&self, gallery: &EhGallery) -> Result<()> {
        info!("重新扫描页面中");
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        for page in &gallery.pages {
            // 如果新表中能查到，则不需要扫描
            if let Some(image) = ImageEntity::get_by_hash(page.hash()).await? {
                PageEntity::create(page.gallery_id(), page.page(), image.id).await?;
                continue;
            };
            // 存在旧 telegraph 上传记录的页面，不需要上传，只需要记录 fileindex 和 page
            if let Some(telegraph) = ImageEntity::get_old_url_by_hash(page.hash()).await? {
                let (fileindex, _) = self.ehentai.get_image_url(page).await?;
                ImageEntity::create(fileindex, page.hash(), &telegraph).await?;
                PageEntity::create(page.gallery_id(), page.page(), fileindex).await?;
            // 否则下载并重新上传
            } else {
                let (fileindex, bytes) = self.ehentai.get_image_bytes(page).await?;
                let resp = Telegraph::upload_with(&[ImageBytes(bytes)], &client).await?;
                ImageEntity::create(fileindex, page.hash(), &resp[0].src).await?;
                PageEntity::create(page.gallery_id(), page.page(), fileindex).await?;
            }
            time::sleep(Duration::from_secs(5)).await;
        }

        Ok(())
    }
}
