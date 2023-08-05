use anyhow::Result;
use teloxide::prelude::Requester;
use teloxide::types::{ChatJoinRequest, ChatMemberKind};
use tracing::info;

use crate::bot::Bot;
use crate::config::Config;
use crate::database::InviteLink;

pub async fn join_request_handler(bot: Bot, jq: ChatJoinRequest, cfg: Config) -> Result<()> {
    // 没有加入群组
    if matches!(
        bot.get_chat_member(cfg.telegram.auth_group_id, jq.from.id).await?.kind,
        ChatMemberKind::Restricted(_) | ChatMemberKind::Banned(_) | ChatMemberKind::Left
    ) {
        info!("{}: 拒绝来自 {} 的加入请求", jq.chat.id, jq.from.id);
        bot.decline_chat_join_request(jq.chat.id, jq.from.id).await?;
        return Ok(());
    }

    if let Some(invite_link) = jq.invite_link {
        InviteLink::create(jq.from.id.0 as i64, &invite_link.invite_link).await?;
    }

    info!("{}: 批准来自 {} 的加入请求", jq.chat.id, jq.from.id);
    bot.approve_chat_join_request(jq.chat.id, jq.from.id).await?;

    Ok(())
}
