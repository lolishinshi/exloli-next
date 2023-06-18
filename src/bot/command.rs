use teloxide::utils::command::BotCommands;

// NOTE: 此处必须实现 Clone，否则不满足 dptree 的 Injectable 约束
#[derive(BotCommands, Clone, PartialEq, Debug)]
#[command(rename_rule = "lowercase")]
pub enum AdminCommand {
    #[command(description = "根据 E 站 URL 上传一个指定画廊，如果已存在，则重新上传")]
    Upload(String),
    #[command(description = "删除所回复的画廊")]
    Delete,
}

#[derive(BotCommands, Clone, PartialEq, Debug)]
#[command(rename_rule = "lowercase")]
pub enum PublicCommand {
    #[command(description = "根据消息 URL 更新一个指定画廊")]
    Update(String),
    #[command(description = "根据 E 站 URL 查询一个指定画廊")]
    Query(String),
    #[command(
        description = "查询从最近 $1 天到 $2 天内的本子排名",
        parse_with = "split"
    )]
    Best(u16, u16),
    #[command(description = "想和本 bot 斗斗吗？")]
    Challenge,
    #[command(description = "pong~")]
    Ping,
}
