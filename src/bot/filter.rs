use teloxide::dispatching::DpHandlerDescription;
use teloxide::prelude::*;
use teloxide::types::ChatMemberKind;

use crate::bot::Bot;
use crate::config::Config;

pub fn filter_admin_msg<Output>() -> Handler<'static, DependencyMap, Output, DpHandlerDescription>
where
    Output: Send + Sync + 'static,
{
    dptree::filter_async(|message: Message, bot: Bot, cfg: Config| async move {
        cfg.telegram.group_id == message.chat.id
            && bot
                .get_chat_member(message.chat.id, message.from().unwrap().id)
                .await
                .map(|member| {
                    matches!(
                        member.kind,
                        ChatMemberKind::Administrator(_) | ChatMemberKind::Owner(_)
                    )
                })
                .unwrap_or_default()
    })
}

pub fn filter_channel_msg<Output>() -> Handler<'static, DependencyMap, Output, DpHandlerDescription>
where
    Output: Send + Sync + 'static,
{
    dptree::filter(|message: Message, cfg: Config| {
        message.from().map(|u| u.id.0 == 777000).unwrap_or_default()
            && message.text().map(|s| s.contains("原始地址")).unwrap_or_default()
            && cfg.telegram.group_id == message.chat.id
    })
}
