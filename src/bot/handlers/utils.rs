use anyhow::Result;
use chrono::{Duration, Utc};
use reqwest::Url;
use teloxide::prelude::*;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, MessageId, Recipient,
};
use teloxide::utils::html::link;

use crate::bot::utils::CallbackData;
use crate::database::GalleryEntity;

pub async fn cmd_best_text(
    start: i32,
    end: i32,
    offset: i32,
    channel: Recipient,
) -> Result<String> {
    let start = Utc::now().date_naive() - Duration::days(start as i64);
    let end = Utc::now().date_naive() - Duration::days(end as i64);
    let text = GalleryEntity::list(start, end, 20, offset)
        .await?
        .iter()
        .map(|(score, title, msgid)| {
            format!(
                "<code>{:.2}</code> - {}",
                score * 100.,
                link(url_of(channel.clone(), *msgid).as_str(), &title),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    return Ok(format!("最近 {} ~ {} 天的本子排名（{}）\n", start, end, offset) + &text);
}

pub fn cmd_best_keyboard(from: i32, to: i32, offset: i32) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::new(
            "<",
            InlineKeyboardButtonKind::CallbackData(CallbackData::PrevPage(from, to, offset).pack()),
        ),
        InlineKeyboardButton::new(
            ">",
            InlineKeyboardButtonKind::CallbackData(CallbackData::NextPage(from, to, offset).pack()),
        ),
    ]])
}

pub fn url_of(channel: Recipient, id: i32) -> Url {
    match channel {
        Recipient::Id(chat_id) => Message::url_of(chat_id, None, MessageId(id)).unwrap(),
        Recipient::ChannelUsername(username) => {
            Message::url_of(ChatId(0), Some(&username[1..]), MessageId(id)).unwrap()
        }
    }
}

pub fn poll_keyboard(poll_id: i32, votes: &[i32; 5]) -> InlineKeyboardMarkup {
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
                InlineKeyboardButtonKind::CallbackData(
                    CallbackData::VoteForPoll(poll_id, (idx + 1) as i32).pack(),
                ),
            )]
        })
        .collect::<Vec<_>>();

    InlineKeyboardMarkup::new(options)
}
