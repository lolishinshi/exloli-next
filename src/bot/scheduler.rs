use std::time::Duration;

use teloxide::prelude::*;
use teloxide::types::MessageId;

use crate::bot::Bot;

#[derive(Debug, Clone)]
pub struct Scheduler {
    bot: Bot,
}

impl Scheduler {
    pub fn new(bot: Bot) -> Self {
        Self { bot }
    }

    pub fn delete_msg(&self, chat_id: ChatId, msg_id: MessageId, seconds: u64) {
        let bot = self.bot.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(seconds)).await;
            let _ = bot.delete_message(chat_id, msg_id).await;
        });
    }
}
