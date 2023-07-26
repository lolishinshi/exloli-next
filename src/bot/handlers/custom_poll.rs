use anyhow::{Context, Result};
use teloxide::prelude::*;
use tracing::info;

use crate::bot::handlers::utils;
use crate::bot::Bot;
use crate::database::{GalleryEntity, PollEntity};
use crate::reply_to;

pub async fn custom_pool_sender(bot: Bot, message: Message) -> Result<()> {
    info!("频道消息更新，发送投票");
    // 辣鸡 tg 安卓客户端在置顶消息过多时似乎在进群时会卡住
    // 因此取消置顶频道自动转发的消息
    bot.unpin_chat_message(message.chat.id).message_id(message.id).await?;

    let msg_id = message.forward_from_message_id().context("找不到消息")?;
    let gallery = GalleryEntity::get_by_msg(msg_id).await?.context("找不到画廊")?;

    // FIXME: 此处如果父画廊还没有记录，则无法找到投票，应该改成不断向 E 站请求父画廊直到有父画廊存在投票或者没有父画廊为止
    // 对于投票的 ID，如果该画廊有投票，则使用该画廊的投票 ID
    let poll_id = match PollEntity::get_by_gallery(gallery.id).await? {
        Some(v) => v.id,
        // 如果没有，则尝试使用其父画廊的投票 ID
        None => match gallery.parent {
            Some(id) => match PollEntity::get_by_gallery(id).await? {
                Some(v) => v.id,
                // 如果还是没有，则使用其画廊 ID
                None => gallery.id as i64,
            },
            None => gallery.id as i64,
        },
    };

    // 此处存在重复插入，但可以忽略
    PollEntity::create(poll_id, gallery.id).await?;

    let votes = PollEntity::get_vote(poll_id).await?;
    let markup = utils::poll_keyboard(poll_id as i32, &votes);

    let score = PollEntity::update_score(poll_id).await? * 100.;
    let sum = votes.iter().sum::<i32>();
    reply_to!(bot, message, format!("当前 {sum} 人投票，{score:.2} 分"))
        .reply_markup(markup)
        .await?;

    Ok(())
}
