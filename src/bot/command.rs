use teloxide::utils::command::BotCommands;

use crate::ehentai::EhGalleryUrl;

// NOTE: 此处必须实现 Clone，否则不满足 dptree 的 Injectable 约束
#[derive(BotCommands, Clone, PartialEq, Debug)]
#[command(rename_rule = "lowercase")]
pub enum AdminCommand {
    #[command(description = "根据 E 站 URL 上传一个指定画廊，如果已存在，则重新上传")]
    Upload(EhGalleryUrl),
    #[command(description = "删除所回复的画廊")]
    Delete,
    #[command(description = "完全删除所回复的画廊，会导致重新上传")]
    Erase,
    // TODO: 该功能需要移除
    #[command(description = "重新扫描排名 $1 ~ $2 本子的页面", parse_with = "split")]
    ReScan(usize, usize),
}

#[derive(BotCommands, Clone, PartialEq, Debug)]
#[command(rename_rule = "lowercase")]
pub enum PublicCommand {
    #[command(description = "根据消息 URL 更新一个指定画廊")]
    Update(String),
    #[command(description = "根据 E 站 URL 查询一个指定画廊")]
    Query(EhGalleryUrl),
    #[command(
        description = "查询从最近 $1 天到 $2 天内的本子排名（$1 < $2）",
        parse_with = "split"
    )]
    Best(u16, u16),
    #[command(description = "想和本 bot 斗斗吗？")]
    Challenge,
    #[command(description = "pong~")]
    Ping,
}
