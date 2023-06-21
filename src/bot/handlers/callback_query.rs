use anyhow::Result;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use tracing::info;

use crate::bot::handlers::{cmd_best_keyboard, cmd_best_text, poll_keyboard};
use crate::bot::utils::{CallbackData, RateLimiter};
use crate::bot::Bot;
use crate::config::Config;
use crate::database::{PollEntity, VoteEntity};

pub fn callback_query_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription>
{
    dptree::entry()
        .branch(case![CallbackData::VoteForPoll(poll, option)].endpoint(callback_vote_for_poll))
        .endpoint(callback_change_page)
}

async fn callback_vote_for_poll(
    bot: Bot,
    query: CallbackQuery,
    limiter: RateLimiter,
    (poll, option): (i32, i32),
) -> Result<()> {
    if let Some(d) = limiter.insert(query.from.id) {
        bot.answer_callback_query(query.id)
            .text(format!("操作频率过高，请等待 {} 秒后再试", d.as_secs()))
            .show_alert(true)
            .await?;
        return Ok(());
    }

    info!("用户投票：[{}] {} = {}", query.from.id, poll, option);

    let old_votes = PollEntity::get_vote(poll as i64).await?;
    VoteEntity::create(query.from.id.0, poll as i64, option).await?;
    let votes = PollEntity::get_vote(poll as i64).await?;

    // 投票没有变化时不要更新，不然会报错 MessageNotModified
    if old_votes == votes {
        return Ok(());
    }

    let score = PollEntity::update_score(poll as i64).await?;
    let sum = votes.iter().sum::<i32>();
    let keyboard = poll_keyboard(poll, &votes);
    let text = format!("当前 {} 人投票，{:.2} 分", sum, score * 100.);

    if let Some(message) = query.message {
        bot.edit_message_text(message.chat.id, message.id, text).reply_markup(keyboard).await?;
    }

    Ok(())
}

async fn callback_change_page(
    bot: Bot,
    query: CallbackQuery,
    callback: CallbackData,
    cfg: Config,
) -> Result<()> {
    let (from, to, offset) = match callback {
        CallbackData::PrevPage(from, to, offset) => (from, to, offset - 1),
        CallbackData::NextPage(from, to, offset) => (from, to, offset + 1),
        _ => unreachable!(),
    };
    let text = cmd_best_text(from, to, offset, cfg.telegram.channel_id).await?;
    let keyboard = cmd_best_keyboard(from, to, offset);

    if let Some(message) = query.message {
        bot.edit_message_text(message.chat.id, message.id, text).reply_markup(keyboard).await?;
    }

    Ok(())
}
