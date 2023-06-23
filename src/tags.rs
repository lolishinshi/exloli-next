use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::Deserialize;
use tokio::time::{self, Duration};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct EhTagTransDB {
    file: String,
    db: Arc<RwLock<Option<EhTagTransDBInner>>>,
}

#[derive(Debug, Deserialize)]
struct EhTagTransDBInner {
    // repo: String,
    // head: Value,
    // version: u8,
    data: Vec<EhTagTransData>,
}

#[derive(Debug, Deserialize)]
struct EhTagTransData {
    namespace: String,
    // frontMatters: Value,
    // count: i32,
    data: HashMap<String, TagInfo>,
}

#[derive(Debug, Deserialize)]
struct TagInfo {
    name: String,
    // intro: String,
    // links: String,
}

impl EhTagTransDB {
    pub fn new(file: &str) -> Self {
        let text = fs::read_to_string(file).expect("无法打开 db.text.json");
        let db = serde_json::from_str(&text).expect("无法解析翻译数据库");
        Self { file: file.to_string(), db: Arc::new(RwLock::new(Some(db))) }
    }

    pub async fn start(&self) {
        loop {
            if let Err(err) = self.update().await {
                error!("更新 tag 错误：{}", err);
            }
            info!("tag 更新完成，等待 10 小时");
            time::sleep(Duration::from_secs(36000)).await;
        }
    }

    async fn update(&self) -> Result<()> {
        info!("更新 tag 中……");
        // 此处得设置 user-agent，否则 github 会 403
        let client = reqwest::Client::builder().user_agent("exloli").build()?;
        let resp = client
            .get("https://api.github.com/repos/EhTagTranslation/Database/releases/latest")
            .send()
            .await?
            .error_for_status()?;
        let json = resp.json::<serde_json::Value>().await?;

        let extract = |v: &serde_json::Value| -> Option<String> {
            for v in v.get("assets")?.as_array()? {
                if v.get("name")?.as_str()? == "db.text.json" {
                    return Some(v.get("browser_download_url")?.as_str()?.to_string());
                }
            }
            None
        };
        let url = extract(&json).context("找不到 URL")?;

        // NOTE: 其实可以检测一下再决定是否需要更新，但是这么点东西懒得检测了
        let resp = client.get(url).send().await?.error_for_status()?;
        let text = resp.text().await?;
        fs::write(&self.file, &text)?;

        let db = serde_json::from_str(&text)?;
        let mut lock = self.db.write().unwrap();
        lock.replace(db);

        Ok(())
    }

    /// 返回不经过任何修改的翻译结果，即多个结果之间用 | 分隔
    pub fn trans_raw(&self, namespace: &str, name: &str) -> String {
        // NOTE: 对于形如 nekogen | miyauchi takeshi 的 tag，只需要取第一部分翻译
        let name = name.split(" | ").next().unwrap();
        let lock = self.db.read().unwrap();
        for ns in &lock.as_ref().unwrap().data {
            if ns.namespace == namespace {
                return ns.data.get(name).map(|info| info.name.as_str()).unwrap_or(name).to_owned();
            }
        }
        name.to_owned()
    }

    /// 根据 namespace 和 tag name 进行翻译
    ///
    /// 可能会返回多个翻译结果
    pub fn trans(&self, namespace: &str, name: &str) -> Vec<String> {
        self.trans_raw(namespace, name).split(" | ").map(|s| s.to_owned()).collect::<Vec<_>>()
    }

    /// 翻译 namespace
    pub fn trans_namespace(&self, namespace: &str) -> String {
        self.trans("rows", namespace).swap_remove(0)
    }

    /// 翻译整组 tags
    pub fn trans_tags(
        &self,
        tags: &IndexMap<String, Vec<String>>,
    ) -> IndexMap<String, Vec<String>> {
        let mut result = IndexMap::new();
        for (namespace, tags) in tags.iter() {
            let t_ns = self.trans_namespace(namespace);
            let t_tags = tags.iter().flat_map(|t| self.trans(namespace, t)).collect::<Vec<_>>();
            result.insert(t_ns.to_owned(), t_tags);
        }
        result
    }
}

#[cfg(test)]
mod test {
    use super::EhTagTransDB;

    #[test]
    fn test() {
        let db = EhTagTransDB::new("./db.text.json");
        assert_eq!(db.trans_namespace("female"), "女性");
        assert_eq!(db.trans("female", "lolicon"), vec!["萝莉"]);
        assert_eq!(db.trans("character", "yui"), vec!["由依", "结衣"]);
    }
}
