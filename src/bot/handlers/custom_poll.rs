use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup};
use tracing::info;

use crate::database::{GalleryEntity, MessageEntity, PollEntity};
use crate::reply_to;

pub async fn channel_msg_handler(bot: Bot, message: Message) -> Result<()> {
    info!("频道消息更新，发送投票");
    // 辣鸡 tg 安卓客户端在置顶消息过多时似乎在进群时会卡住
    // 因此取消置顶频道自动转发的消息
    bot.unpin_chat_message(message.chat.id).message_id(message.id).await?;

    let msg_id = message.forward_from_message_id().unwrap();
    let gallery = GalleryEntity::get_by_msg(msg_id).await?.unwrap();

    // 对于投票的 ID，如果该画廊有投票，则使用该画廊的投票 ID
    let poll_id = match PollEntity::get_by_gallery(gallery.id).await? {
        Some(v) => v.id,
        // 如果没有，则尝试使用其父画廊的投票 ID
        None => match gallery.parent {
            Some(id) => match PollEntity::get_by_gallery(id).await? {
                Some(v) => v.id,
                // 如果还是没有，则使用其消息 ID
                None => msg_id as i64,
            },
            None => msg_id as i64,
        },
    };

    // 此处存在重复插入，但可以忽略
    PollEntity::create(poll_id, gallery.id).await?;

    let votes = PollEntity::get_vote(poll_id).await?;
    let markup = poll_keyboard(poll_id as i32, &votes);

    reply_to!(bot, message, "当前 0 人投票，0.00 分").reply_markup(markup).await?;

    Ok(())
}

fn poll_keyboard(poll_id: i32, votes: &[i32; 5]) -> InlineKeyboardMarkup {
    let sum = votes.iter().sum::<i32>();
    let votes: Box<dyn Iterator<Item = f32>> = if sum == 0 {
        Box::new([0.].iter().cloned().cycle())
    } else {
        Box::new(votes.iter().map(|&i| i as f32 / sum as f32 * 100.))
    };

    let options = ["我瞎了", "不咋样", "还行吧", "不错哦", "太棒了"]
        .iter()
        .zip(votes)
        .enumerate()
        .map(|(idx, (name, vote))| {
            vec![InlineKeyboardButton::new(
                format!("{:.0}% {}", vote, name),
                InlineKeyboardButtonKind::CallbackData(format!("vote {} {}", poll_id, idx + 1)),
            )]
        })
        .collect::<Vec<_>>();

    InlineKeyboardMarkup::new(options)
}
