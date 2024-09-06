use std::borrow::Cow;

pub mod html;

/// 左填充空格
pub fn pad_left(s: &str, len: usize) -> Cow<str> {
    let width = unicode_width::UnicodeWidthStr::width(s);
    if width >= len {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(" ".repeat(len - width) + s)
    }
}
