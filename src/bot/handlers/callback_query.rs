use anyhow::{Context, Result};
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::utils::html::{link, user_mention};
use tracing::info;

use super::utils::gallery_preview_url;
use crate::bot::handlers::{cmd_best_keyboard, cmd_best_text, poll_keyboard};
use crate::bot::utils::{CallbackData, ChallengeLocker, RateLimiter};
use crate::bot::Bot;
use crate::config::Config;
use crate::database::{ChallengeHistory, GalleryEntity, PollEntity, VoteEntity};
use crate::ehentai::GalleryInfo;
use crate::tags::EhTagTransDB;

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
        let gallery_entity = GalleryEntity::get(gallery).await?.context("找不到画廊")?;
        let preview = gallery_preview_url(cfg.telegram.channel_id, gallery).await?;
        let poll = PollEntity::get_by_gallery(gallery).await?.context("找不到投票")?;
        ChallengeHistory::create(query.from.id.0 as i64, gallery, page, success, message.chat.id.0)
            .await?;

        let mention = user_mention(query.from.id.0 as i64, &query.from.full_name());
        let result = if success { "答对了！" } else { "答错了……" };
        let artist = trans.trans_raw("artist", &answer);
        let url = gallery_entity.url().url();
        let preview = link(&preview, &gallery_entity.title_jp.unwrap_or(gallery_entity.title));
        let score = poll.score * 100.;

        let text = format!(
            "{mention} {result}，答案是 {artist}（{answer}）\n地址：{url}\n预览：{preview}\n评分：{score:.2}",
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
        bot.edit_message_text(message.chat.id, message.id, text)
            .reply_markup(keyboard)
            .disable_web_page_preview(true)
            .await?;
    }

    Ok(())
}
