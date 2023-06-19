use std::fmt::Debug;
use std::time::Duration;

use futures::prelude::*;
use indexmap::IndexMap;
use reqwest::header::*;
use reqwest::Client;
use serde::Serialize;
use tracing::{debug, error};

use super::error::*;
use super::types::*;
use crate::utils::xpath::Node;

macro_rules! headers {
    ($($k:ident => $v:expr), *) => {{
        [
            $(($k.clone(), $v.parse().unwrap()),)*
        ].into_iter().collect::<HeaderMap>()
    }};
}

macro_rules! send {
    ($e:expr) => {
        $e.send()
            .await
            .and_then(reqwest::Response::error_for_status)
    };
}

#[derive(Debug, Clone)]
pub struct EhClient(Client);

impl EhClient {
    #[tracing::instrument]
    pub async fn new(cookie: &str) -> Result<Self> {
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
    #[tracing::instrument(skip(self))]
    pub async fn search_skip<T: Serialize + ?Sized + Debug>(
        &self,
        params: &T,
        next: i32,
    ) -> Result<Vec<EhGalleryUrl>> {
        let resp = send!(self
            .0
            .get("https://exhentai.org")
            .query(params)
            .query(&[("next", next)]))?;
        let html = Node::from_html(&resp.text().await?)?;

        let gl_list = html.xpath_elem(r#"//table[@class="itg gltc"]/tr[position() > 1]"#)?;

        let mut ret = vec![];
        for gl in gl_list {
            let title = gl.xpath_text(r#".//td[@class="gl3c glname"]/a/div/text()"#)?;
            let url = gl.xpath_text(r#".//td[@class="gl3c glname"]/a/@href"#)?;
            debug!(url, title);
            ret.push(url.parse()?)
        }

        Ok(ret)
    }

    /// 搜索前 N 页的本子，返回一个异步迭代器
    #[tracing::instrument(skip(self))]
    pub fn search_iter<'a, T: Serialize + ?Sized + Debug>(
        &'a self,
        params: &'a T,
        page: usize,
    ) -> impl Stream<Item = EhGalleryUrl> + 'a {
        stream::unfold(0, move |next| async move {
            match self.search_skip(params, next).await {
                Ok(gls) => {
                    let next = gls.last().unwrap().id();
                    Some((stream::iter(gls), next))
                }
                Err(e) => {
                    error!("search error: {}", e);
                    None
                }
            }
        })
        .take(page)
        .flatten()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_gallery(&self, url: &EhGalleryUrl) -> Result<EhGallery> {
        let resp = send!(self.0.get(url.url()))?;
        let mut html = Node::from_html(&resp.text().await?)?;

        // 英文标题、日文标题、父画廊
        let title = html.xpath_text(r#"//h1[@id="gn"]/text()"#)?;
        let title_jp = html.xpath_text(r#"//h1[@id="gj"]/text()"#).ok();
        let parent = html
            .xpath_text(r#"//tr[contains(./td[1]/text(), "Parent:")]/td[2]/a/@href"#)
            .ok();

        // 画廊 tag
        let mut tags = IndexMap::new();
        for ele in html
            .xpath_elem(r#"//div[@id="taglist"]//tr"#)
            .unwrap_or_default()
        {
            let tag_set_name = ele
                .xpath_text(r#"./td[1]/text()"#)?
                .trim_matches(':')
                .to_owned();
            let tag = ele.xpath_texts(r#"./td[2]/div/a/text()"#)?;
            tags.insert(tag_set_name, tag);
        }

        // 每一页的 URL
        let mut pages = html.xpath_texts(r#"//div[@id="gdt"]//a/@href"#)?;
        while let Ok(next_page) = html.xpath_text(r#"//table[@class="ptt"]//td[last()]/a/@href"#) {
            debug!(next_page);
            let resp = send!(self.0.get(&next_page))?;
            html = Node::from_html(&resp.text().await?)?;
            pages.extend(html.xpath_texts(r#"//div[@id="gdt"]//a/@href"#)?);
        }

        let pages = pages.into_iter().map(EhPageUrl::new).collect();

        Ok(EhGallery {
            url: url.clone(),
            title,
            title_jp,
            parent,
            tags,
            pages,
        })
    }

    /// 获取画廊的某一页的图片实际地址
    #[tracing::instrument(skip(self))]
    pub async fn get_image_url(&self, page: &EhPageUrl) -> Result<String> {
        let resp = send!(self.0.get(page.url()))?;
        let html = Node::from_html(&resp.text().await?)?;
        let img = html.xpath_text(r#"//img[@id="img"]/@src"#)?;
        Ok(img)
    }

    /// 获取画廊的某一页的图片的字节流
    #[tracing::instrument(skip(self))]
    pub async fn get_image_bytes(&self, page: &EhPageUrl) -> Result<Vec<u8>> {
        let url = self.get_image_url(page).await?;
        let resp = send!(self.0.get(url))?;
        Ok(resp.bytes().await?.to_vec())
    }
}
