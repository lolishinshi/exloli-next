use anyhow::{Context, Result};
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::utils::html::user_mention;
use tracing::info;

use super::utils::url_of;
use crate::bot::handlers::{cmd_best_keyboard, cmd_best_text, poll_keyboard};
use crate::bot::utils::{CallbackData, ChallengeLocker, RateLimiter};
use crate::bot::Bot;
use crate::config::Config;
use crate::database::{ChallengeHistory, MessageEntity, PollEntity, VoteEntity};
use crate::utils::tags::EhTagTransDB;

pub fn callback_query_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription>
{
    dptree::entry()
        .branch(case![CallbackData::VoteForPoll(poll, option)].endpoint(callback_vote_for_poll))
        .branch(case![CallbackData::Challenge(id, artist)].endpoint(callback_challenge))
        .endpoint(callback_change_page)
}

async fn callback_challenge(
    bot: Bot,
    query: CallbackQuery,
    trans: EhTagTransDB,
    locker: ChallengeLocker,
    cfg: Config,
    (id, artist): (i64, String),
) -> Result<()> {
    let message = query.message.context("消息过旧")?;
    info!("{}: <- challenge {} {}", query.from.id, id, artist);
    if let Some((gallery, page, answer)) = locker.get_challenge(id) {
        let success = answer == artist;
        let msg_entity = MessageEntity::get_by_gallery_id(gallery).await?.context("找不到消息")?;
        let poll = PollEntity::get_by_gallery(gallery).await?.context("找不到投票")?;
        ChallengeHistory::create(query.from.id.0 as i64, gallery, page, success).await?;
        let text = format!(
            "{} {}，答案是 {}（{}）\n消息：{}\n评分：{:.2}",
            user_mention(query.from.id.0 as i64, &query.from.full_name()),
            if success { "答对了！" } else { "答错了……" },
            trans.trans_raw("artist", &answer),
            &answer,
            url_of(cfg.telegram.channel_id, msg_entity.id),
            poll.score * 100.,
        );
        bot.edit_message_caption(message.chat.id, message.id).caption(text).await?;
    }
    Ok(())
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
