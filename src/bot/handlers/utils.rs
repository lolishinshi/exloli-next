use anyhow::Result;
use chrono::{Duration, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::types::{MessageId, Recipient};
use teloxide::utils::html::link;

use crate::database::GalleryEntity;

#[derive(Serialize, Deserialize)]
pub enum CallbackData {
    VoteForPoll(i32, i32),
}

pub async fn cmd_best_text(
    start: i64,
    end: i64,
    offset: i64,
    channel: Recipient,
) -> Result<String> {
    let start = Utc::now().date_naive() - Duration::days(start as i64);
    let end = Utc::now().date_naive() - Duration::days(end as i64);
    let text = GalleryEntity::list(start, end, 20, 1)
        .await?
        .iter()
        .map(|(score, title, msgid)| {
            format!(
                "<code>{:.2}</code> - {}",
                score,
                link(url_of(channel.clone(), *msgid).as_str(), &title),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    return Ok(format!("最近 {} ~ {} 天的本子排名（{}）\n", start, end, offset) + &text);
}

pub fn url_of(channel: Recipient, id: i32) -> Url {
    match channel {
        Recipient::Id(chat_id) => Message::url_of(chat_id, None, MessageId(id)).unwrap(),
        Recipient::ChannelUsername(username) => {
            Message::url_of(ChatId(0), Some(&username[1..]), MessageId(id)).unwrap()
        }
    }
}
