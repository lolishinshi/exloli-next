use std::fmt::Display;
use std::str::FromStr;

use chrono::prelude::*;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;

use super::error::EhError;
use crate::database::GalleryEntity;

// 画廊地址，格式为 https://exhentai.org/g/2549143/16b1b7bab0/
#[derive(Debug, Clone, PartialEq)]
pub struct EhGalleryUrl {
    id: i32,
    token: String,
    cover: usize,
}

impl EhGalleryUrl {
    /// 画廊 URL
    pub fn url(&self) -> String {
        format!("https://exhentai.org/g/{}/{}/", self.id, self.token)
    }

    /// 画廊 ID
    pub fn id(&self) -> i32 {
        self.id
    }

    /// 画廊 token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// 封面是第几张
    pub fn cover(&self) -> usize {
        self.cover
    }
}

impl FromStr for EhGalleryUrl {
    type Err = EhError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"https://e.hentai.org/g/(?P<id>\d+)/(?P<token>[^/]+)/?(?P<cover>#\d+)?")
                .unwrap()
        });
        let captures = RE.captures(s).ok_or_else(|| EhError::InvalidURL(s.to_owned()))?;
        // NOTE: 由于是正则匹配出来的结果，此处 unwrap 不会造成 panic
        let token = captures.name("token").unwrap().as_str().to_owned();
        let id = captures.name("id").and_then(|s| s.as_str().parse().ok()).unwrap();
        let cover =
            captures.name("cover").and_then(|s| s.as_str()[1..].parse().ok()).unwrap_or_default();

        Ok(Self { id, token, cover })
    }
}

impl Display for EhGalleryUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.url())
    }
}

/// 画廊页面地址，格式为 https://exhentai.org/s/03af734602/1932743-1
#[derive(Debug, Clone, PartialEq)]
pub struct EhPageUrl {
    hash: String,
    gallery_id: i32,
    page: i32,
    nl: Option<String>,
}

impl EhPageUrl {
    pub fn url(&self) -> String {
        match &self.nl {
            None => {
                format!("https://exhentai.org/s/{}/{}-{}", self.hash, self.gallery_id, self.page)
            }
            Some(nl) => format!(
                "https://exhentai.org/s/{}/{}-{}?nl={}",
                self.hash, self.gallery_id, self.page, nl
            ),
        }
    }

    /// 页面哈希，实际上就是图片哈希的前十位
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// 画廊 ID
    pub fn gallery_id(&self) -> i32 {
        self.gallery_id
    }

    /// 页码
    pub fn page(&self) -> i32 {
        self.page
    }

    pub fn with_nl(&self, nl: &str) -> Self {
        EhPageUrl {
            hash: self.hash.clone(),
            gallery_id: self.gallery_id,
            page: self.page,
            nl: Some(nl.to_owned()),
        }
    }
}

impl FromStr for EhPageUrl {
    type Err = EhError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"https://e.hentai.org/s/(?P<hash>.+)/(?P<id>\d+)-(?P<page>\d+)").unwrap()
        });

        let captures = RE.captures(s).ok_or_else(|| EhError::InvalidURL(s.to_owned()))?;
        // NOTE: 由于是正则匹配出来的结果，此处 unwrap 不会造成 panic
        let hash = captures.name("hash").unwrap().as_str().to_owned();
        let gallery_id = captures.name("id").and_then(|s| s.as_str().parse().ok()).unwrap();
        let page = captures.name("page").and_then(|s| s.as_str().parse().ok()).unwrap();

        Ok(Self { hash: hash.to_owned(), gallery_id, page, nl: None })
    }
}

impl Display for EhPageUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.url())
    }
}

#[derive(Debug, Clone)]
pub struct EhGallery {
    /// URL
    pub url: EhGalleryUrl,
    /// 画廊标题
    pub title: String,
    /// 画廊日文标题
    pub title_jp: Option<String>,
    /// 画廊标签
    pub tags: IndexMap<String, Vec<String>>,
    /// 收藏数量
    pub favorite: i32,
    /// 父画廊地址
    pub parent: Option<EhGalleryUrl>,
    /// 画廊页面
    pub pages: Vec<EhPageUrl>,
    /// 发布时间
    pub posted: NaiveDateTime,
    /// 封面是第几张
    pub cover: usize,
}

pub trait GalleryInfo {
    fn url(&self) -> EhGalleryUrl;

    fn title(&self) -> String;

    fn title_jp(&self) -> String;

    fn tags(&self) -> &IndexMap<String, Vec<String>>;

    fn pages(&self) -> usize;

    fn cover(&self) -> usize;
}

impl GalleryInfo for EhGallery {
    fn url(&self) -> EhGalleryUrl {
        self.url.clone()
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn title_jp(&self) -> String {
        self.title_jp.clone().unwrap_or_else(|| self.title.clone())
    }

    fn tags(&self) -> &IndexMap<String, Vec<String>> {
        &self.tags
    }

    fn pages(&self) -> usize {
        self.pages.len()
    }

    fn cover(&self) -> usize {
        self.cover
    }
}

impl GalleryInfo for GalleryEntity {
    fn url(&self) -> EhGalleryUrl {
        format!("https://exhentai.org/g/{}/{}", self.id, self.token).parse().unwrap()
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn title_jp(&self) -> String {
        self.title_jp.clone().unwrap_or_else(|| self.title.clone())
    }

    fn tags(&self) -> &IndexMap<String, Vec<String>> {
        &self.tags.0
    }

    fn pages(&self) -> usize {
        self.pages as usize
    }

    fn cover(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_gallery_url() {
        let s = "https://exhentai.org/g/2423705/3962191348/";
        let url = s.parse::<EhGalleryUrl>().unwrap();
        assert_eq!(url.id, 2423705);
        assert_eq!(url.token, "3962191348");
        assert_eq!(url.url(), s);
    }

    #[test]
    fn parse_page_url() {
        let s = "https://exhentai.org/s/03af734602/1932743-1";
        let url = s.parse::<EhPageUrl>().unwrap();
        assert_eq!(url.hash, "03af734602");
        assert_eq!(url.gallery_id, 1932743);
        assert_eq!(url.page, 1);
        assert_eq!(url.url(), s);
    }
}
