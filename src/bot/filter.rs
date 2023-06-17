use teloxide::dispatching::DpHandlerDescription;
use teloxide::prelude::*;
use teloxide::types::ChatMemberKind;

pub fn filter_admin_msg<Output>() -> Handler<'static, DependencyMap, Output, DpHandlerDescription>
where
    Output: Send + Sync + 'static,
{
    dptree::filter_async(|message: Message, bot: Bot| async move {
        bot.get_chat_member(message.chat.id, message.from().unwrap().id)
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
