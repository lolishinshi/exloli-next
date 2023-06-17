use std::{fmt, ops::Deref, rc::Rc};

use libxml::parser::Parser;
use libxml::tree::{self, Document, NodeType};
use libxml::xpath::Context as XContext;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, XpathError>;

#[derive(Debug, Error)]
pub enum XpathError {
    #[error("failed to evaluate xpath: {0}")]
    Evaluate(String),
    #[error("not found: {0}")]
    ElementNotFound(String),
    #[error("failed to parse html: {0}")]
    HTMLParserError(#[from] libxml::parser::XmlParseError),
    #[error("failed to create xml context")]
    LibXMLContextError,
}

#[derive(Debug)]
pub enum Value {
    Element(Vec<Node>),
    Text(Vec<String>),
    None,
}

impl Value {
    pub fn into_element(self) -> Option<Vec<Node>> {
        match self {
            Value::Element(v) => Some(v),
            _ => None,
        }
    }

    pub fn into_text(self) -> Option<Vec<String>> {
        match self {
            Value::Text(v) => Some(v),
            _ => None,
        }
    }
}

pub struct Node {
    document: Rc<Document>,
    context: Rc<XContext>,
    node: tree::Node,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.get_type() {
            Some(NodeType::ElementNode) => {
                write!(f, "<Element {} at {:p}>", self.get_name(), self.node_ptr())
            }
            Some(NodeType::AttributeNode) | Some(NodeType::TextNode) => {
                write!(f, "{:?}", self.get_content())
            }
            _ => unimplemented!(),
        }
    }
}

impl Node {
    pub fn xpath_texts(&self, xpath: &str) -> Result<Vec<String>> {
        self.xpath(xpath)?
            .into_text()
            .ok_or_else(|| XpathError::ElementNotFound(xpath.to_owned()))
    }

    pub fn xpath_text(&self, xpath: &str) -> Result<String> {
        self.xpath_texts(xpath)?
            .into_iter()
            .next()
            .ok_or_else(|| XpathError::ElementNotFound(xpath.to_owned()))
    }

    pub fn xpath_elem(&self, xpath: &str) -> Result<Vec<Node>> {
        self.xpath(xpath)?
            .into_element()
            .ok_or_else(|| XpathError::ElementNotFound(xpath.to_owned()))
    }

    pub fn xpath(&self, xpath: &str) -> Result<Value> {
        let nodes = self
            .context
            .node_evaluate(xpath, &self.node)
            .map_err(|_| XpathError::Evaluate(xpath.to_owned()))?
            .get_nodes_as_vec();
        let result = match nodes.get(0) {
            Some(node) => match node.get_type() {
                Some(NodeType::ElementNode) => Value::Element(
                    nodes
                        .into_iter()
                        .map(|node| Node {
                            document: self.document.clone(),
                            context: self.context.clone(),
                            node,
                        })
                        .collect(),
                ),
                Some(NodeType::AttributeNode) | Some(NodeType::TextNode) => {
                    Value::Text(nodes.into_iter().map(|node| node.get_content()).collect())
                }
                _ => unimplemented!(),
            },
            None => Value::None,
        };
        Ok(result)
    }
}

impl Node {
    pub fn from_html(html: &str) -> Result<Self> {
        let parser = Parser::default_html();
        let document = parser.parse_string(html)?;
        let context = XContext::new(&document).map_err(|_| XpathError::LibXMLContextError)?;
        let root = document
            .get_root_element()
            .ok_or_else(|| XpathError::ElementNotFound("root".to_owned()))?;
        Ok(Node {
            document: Rc::new(document),
            context: Rc::new(context),
            node: root,
        })
    }
}

impl Deref for Node {
    type Target = tree::Node;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

#[cfg(test)]
mod tests {
    use super::Node;

    #[test]
    fn find_nodes() {
        let html = r#"
        <!doctype html>
        <html lang="zh-CN" dir="ltr">
          <head>
            <meta charset="utf-8">
            <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline' resource: chrome:; connect-src https:; img-src https: data: blob:; style-src 'unsafe-inline';">
            <title>新标签页</title>
            <link rel="icon" type="image/png" href="chrome://branding/content/icon32.png"/>
            <link rel="stylesheet" href="chrome://browser/content/contentSearchUI.css" />
            <link rel="stylesheet" href="resource://activity-stream/css/activity-stream.css" />
          </head>
          <body class="activity-stream">
            <div id="root"><!-- Regular React Rendering --></div>
            <div id="snippets-container">
              <div id="snippets"></div>
            </div>
             <table id="wow" class="lol">
              <tr class="head">
                <th>Firstname</th>
                <th>Lastname</th>
                <th>Age</th>
              </tr>
              <tr class="body">
                <td>Jill</td>
                <td>Smith</td>
                <td>50</td>
              </tr>
              <tr class="body">
                <td>Eve</td>
                <td>Jackson</td>
                <td>94</td>
              </tr>
             </table>
          </body>
        </html>
        "#;
        let node = Node::from_html(html).unwrap();
        assert!(node.xpath(r#"//table"#).is_ok());
        assert_eq!(node.xpath_texts(r#"//table/@class"#), Ok(vec!["lol"]));
        assert_eq!(
            node.xpath(r#"//table//th/text()"#),
            Ok(vec!["Firstname", "Lastname", "Age"])
        );

        for td in node.xpath("//td").unwrap().into_element().unwrap() {
            assert!(td.xpath_texts(".//text()").is_ok());
        }
    }
}
