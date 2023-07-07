use std::fmt::Debug;
use std::time::Duration;

use chrono::prelude::*;
use futures::prelude::*;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::*;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use tokio::runtime::Handle;
use tokio::task;
use tracing::{debug, error, info, Instrument};

use super::error::*;
use super::types::*;
use crate::utils::html::SelectorExtend;

macro_rules! headers {
    ($($k:ident => $v:expr), *) => {{
        [
            $(($k.clone(), $v.parse().unwrap()),)*
        ].into_iter().collect::<HeaderMap>()
    }};
}

macro_rules! send {
    ($e:expr) => {
        $e.send().await.and_then(reqwest::Response::error_for_status)
    };
}

macro_rules! selector {
    ($selector:tt) => {
        Selector::parse($selector).unwrap()
    };
}

#[derive(Debug, Clone)]
pub struct EhClient(pub Client);

impl EhClient {
    #[tracing::instrument(skip(cookie))]
    pub async fn new(cookie: &str) -> Result<Self> {
        info!("登陆 E 站中");
        let headers = headers! {
            ACCEPT => "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ACCEPT_ENCODING => "gzip, deflate, br",
            ACCEPT_LANGUAGE => "zh-CN,en-US;q=0.7,en;q=0.3",
            CACHE_CONTROL => "max-age=0",
            CONNECTION => "keep-alive",
            HOST => "exhentai.org",
            REFERER => "https://exhentai.org",
            UPGRADE_INSECURE_REQUESTS => "1",
            USER_AGENT => "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:67.0) Gecko/20100101 Firefox/67.0",
            COOKIE => cookie
        };

        let client = Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()?;

        // 获取必要的 cookie
        let _response = send!(client.get("https://exhentai.org/uconfig.php"))?;
        let _response = send!(client.get("https://exhentai.org/mytags"))?;

        Ok(Self(client))
    }

    /// 使用指定参数查询符合要求的画廊列表
    #[tracing::instrument(skip(self, params))]
    pub async fn search_skip<T: Serialize + ?Sized + Debug>(
        &self,
        params: &T,
        next: i32,
    ) -> Result<Vec<EhGalleryUrl>> {
        let resp =
            send!(self.0.get("https://exhentai.org").query(params).query(&[("next", next)]))?;
        let html = Html::parse_document(&resp.text().await?);

        let selector = selector!("table.itg.gltc tr");
        let gl_list = html.select(&selector);

        let mut ret = vec![];
        // 第一个是 header
        for gl in gl_list.skip(1) {
            let title = gl.select_text("td.gl3c.glname a div.glink").unwrap();
            let url = gl.select_attr("td.gl3c.glname a", "href").unwrap();
            info!(url, title);
            ret.push(url.parse()?)
        }

        Ok(ret)
    }

    /// 搜索前 N 页的本子，返回一个异步迭代器
    #[tracing::instrument(skip(self, params))]
    pub fn search_iter<'a, T: Serialize + ?Sized + Debug>(
        &'a self,
        params: &'a T,
    ) -> impl Stream<Item = EhGalleryUrl> + 'a {
        stream::unfold(0, move |next| {
            async move {
                match self.search_skip(params, next).await {
                    Ok(gls) => {
                        let next = gls.last().unwrap().id();
                        info!("下一页 {}", next);
                        Some((stream::iter(gls), next))
                    }
                    Err(e) => {
                        error!("search error: {}", e);
                        None
                    }
                }
            }
            .in_current_span()
        })
        .flatten()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_gallery(&self, url: &EhGalleryUrl) -> Result<EhGallery> {
        let resp = send!(self.0.get(url.url()))?;
        let mut html = Html::parse_document(&resp.text().await?);

        // 英文标题、日文标题、父画廊
        let title = html.select_text("h1#gn").unwrap();
        let title_jp = html.select_text("h1#gj");
        let parent = html.select_attr("td.gdt2 a", "href").and_then(|s| s.parse().ok());

        // 画廊 tag
        let mut tags = IndexMap::new();
        let selector = selector!("div#taglist tr");
        for ele in html.select(&selector) {
            let namespace = ele.select_text("td.tc").unwrap().trim_matches(':').to_string();
            let tag = ele.select_texts("td div a");
            tags.insert(namespace, tag);
        }

        // 收藏数量
        let favorite = html.select_text("#favcount").unwrap();
        let favorite = favorite.split(' ').next().unwrap().parse().unwrap();

        // 发布时间
        let posted = &html.select_texts("td.gdt2")[0];
        let posted = NaiveDateTime::parse_from_str(posted, "%Y-%m-%d %H:%M")?;

        // 每一页的 URL
        let mut pages = html.select_attrs("div.gdtl a", "href");
        while let Some(next_page) = html.select_attr("table.ptt td:last-child a", "href") {
            debug!(next_page);
            // FIXME: 此处的迷惑行为，不知道有没有更好的解决办法
            // 1. 由于 Html 结构体是 !Send 的，我们要避免在它被包含在 Future 的上下文中，否则这个 Future
            //    也会是 !Send，然后就没办法 tokio::spawn 了（不过可以 tokio::task::spawn_local
            // 2. 使用 block_on 可以让这个 Future 立即被消耗掉，但是这样会阻塞当前线程，所以需要在外面
            //    再套上 block_in_place
            let client = self.0.clone();
            let text = task::block_in_place(move || {
                Handle::current().block_on(async move {
                    let resp = send!(client.get(next_page))?;
                    Result::Ok(resp.text().await?)
                })
            })?;
            html = Html::parse_document(&text);
            pages.extend(html.select_attrs("div.gdtm a", "href"));
        }

        let pages = pages.into_iter().map(|s| s.parse()).collect::<Result<Vec<_>>>()?;

        Ok(EhGallery { url: url.clone(), title, title_jp, parent, tags, favorite, pages, posted })
    }

    /// 获取画廊的某一页的图片实际地址
    #[tracing::instrument(skip(self))]
    pub async fn get_image_url(&self, page: &EhPageUrl) -> Result<String> {
        let resp = send!(self.0.get(page.url()))?;
        let html = Html::parse_document(&resp.text().await?);
        Ok(html.select_attr("img#img", "src").unwrap())
    }

    /// 获取画廊的某一页的图片的 fileindex 和字节流
    #[tracing::instrument(skip(self))]
    pub async fn get_image_bytes(&self, page: &EhPageUrl) -> Result<(u32, Vec<u8>)> {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"fileindex=(?P<fileindex>\d+)").unwrap());

        let url = self.get_image_url(page).await?;

        let captures = RE.captures(&url).unwrap();
        let fileindex = captures.name("fileindex").unwrap().as_str().parse().unwrap();

        let resp = send!(self.0.get(url))?;
        Ok((fileindex, resp.bytes().await?.to_vec()))
    }
}
