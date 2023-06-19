use std::str::FromStr;

use indexmap::IndexMap;

use super::error::EhError;

// 画廊地址，格式为 https://e-hentai.org/g/2549143/16b1b7bab0/
#[derive(Debug, Clone, PartialEq)]
pub struct EhGalleryUrl(pub(super) String);

impl EhGalleryUrl {
    /// 画廊 URL
    pub fn url(&self) -> &str {
        &self.0
    }

    /// 画廊 ID
    pub fn id(&self) -> i32 {
        self.0.split('/').nth(4).unwrap().parse().unwrap()
    }

    /// 画廊 token
    pub fn token(&self) -> &str {
        self.0.split('/').nth(5).unwrap()
    }
}

impl FromStr for EhGalleryUrl {
    type Err = EhError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("https://exhentai.org/g/") {
            Ok(Self(s.to_owned()))
        } else {
            Err(EhError::InvalidURL(s.to_owned()))
        }
    }
}

/// 画廊页面地址，格式为 https://exhentai.org/s/03af734602/1932743-1
#[derive(Debug, Clone, PartialEq)]
pub struct EhPageUrl(pub(super) String);

impl EhPageUrl {
    pub fn url(&self) -> &str {
        &self.0
    }

    /// 页面哈希，实际上就是图片哈希的前十位
    pub fn hash(&self) -> &str {
        self.0.split('/').nth(4).unwrap()
    }

    /// 画廊 ID
    pub fn gallery_id(&self) -> i32 {
        let last = self.0.split('/').last().unwrap();
        last.split('-').next().unwrap().parse().unwrap()
    }

    /// 页码
    pub fn page(&self) -> i32 {
        let last = self.0.split('/').last().unwrap();
        last.split('-').nth(1).unwrap().parse().unwrap()
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
    /// 父画廊地址
    pub parent: Option<EhGalleryUrl>,
    /// 画廊页面
    pub pages: Vec<EhPageUrl>,
}
