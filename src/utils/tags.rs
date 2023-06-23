use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct EhTagTransDB(Arc<EhTagTransDBInner>);

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
        let text = std::fs::read_to_string(file).expect("无法打开 db.text.json");
        Self(Arc::new(serde_json::from_str(&text).expect("无法解析翻译数据库")))
    }

    /// 返回不经过任何修改的翻译结果，即多个结果之间用 | 分隔
    pub fn trans_raw<'a>(&'a self, namespace: &str, name: &'a str) -> &'a str {
        // NOTE: 对于形如 nekogen | miyauchi takeshi 的 tag，只需要取第一部分翻译
        let name = name.split(" | ").next().unwrap();
        for ns in &self.0.data {
            if ns.namespace == namespace {
                return ns.data.get(name).map(|info| info.name.as_str()).unwrap_or(name);
            }
        }
        name
    }

    /// 根据 namespace 和 tag name 进行翻译
    ///
    /// 可能会返回多个翻译结果
    pub fn trans<'s>(&'s self, namespace: &str, name: &'s str) -> Vec<&'s str> {
        self.trans_raw(namespace, name).split(" | ").collect::<Vec<_>>()
    }

    /// 翻译 namespace
    pub fn trans_namespace<'s>(&'s self, namespace: &'s str) -> &'s str {
        self.trans("rows", namespace)[0]
    }

    /// 翻译整组 tags
    pub fn trans_tags(
        &self,
        tags: &IndexMap<String, Vec<String>>,
    ) -> IndexMap<String, Vec<String>> {
        let mut result = IndexMap::new();
        for (namespace, tags) in tags.iter() {
            let t_ns = self.trans_namespace(namespace);
            let t_tags = tags
                .iter()
                .flat_map(|t| self.trans(namespace, t))
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
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
