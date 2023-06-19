use std::collections::HashMap;

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EhTagTransDB {
    // repo: String,
    // head: Value,
    // version: u8,
    data: Vec<EhTagTransData>,
}

#[derive(Debug, Deserialize)]
pub struct EhTagTransData {
    namespace: String,
    // frontMatters: Value,
    // count: i32,
    data: HashMap<String, TagInfo>,
}

#[derive(Debug, Deserialize)]
pub struct TagInfo {
    name: String,
    // intro: String,
    // links: String,
}

impl EhTagTransDB {
    pub fn new(file: &str) -> Self {
        let text = std::fs::read_to_string(file).expect("无法打开 db.text.json");
        serde_json::from_str(&text).expect("无法解析翻译数据库")
    }

    /// 根据 namespace 和 tag name 进行翻译
    ///
    /// 可能会返回多个翻译结果
    pub fn trans<'s>(&'s self, namespace: &str, name: &'s str) -> Vec<&'s str> {
        for ns in &self.data {
            if ns.namespace == namespace {
                let result = ns
                    .data
                    .get(name)
                    .map(|info| info.name.as_str())
                    .unwrap_or(name);
                return result.split(" | ").collect::<Vec<_>>();
            }
        }
        vec![name]
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
                .map(|t| self.trans(namespace, t))
                .flatten()
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
