#[derive(Debug)]
pub struct SearchResultGallery {
    /// 画廊标题
    pub title: String,
    /// 画廊地址
    pub url: String,
}

pub trait GalleryInfo {
    /// 画廊地址
    fn url(&self) -> &str;

    /// 画廊 ID
    fn id(&self) -> i32 {
        // 地址格式为 https://e-hentai.org/g/2549143/16b1b7bab0/
        self.url().split('/').nth(4).unwrap().parse().unwrap()
    }

    /// 画廊 token
    fn token(&self) -> &str {
        self.url().split('/').nth(5).unwrap()
    }
}

impl GalleryInfo for SearchResultGallery {
    fn url(&self) -> &str {
        &self.url
    }
}

#[derive(Debug)]
pub struct EHGallery {
    /// 画廊标题
    pub title: String,
    /// 画廊日文标题
    pub title_jp: Option<String>,
    /// 画廊标签
    pub tags: Vec<(String, Vec<String>)>,
    /// 父画廊地址
    pub parent: Option<String>,
    /// 画廊页面
    pub pages: Vec<String>,
}
