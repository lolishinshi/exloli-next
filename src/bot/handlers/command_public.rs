use anyhow::{anyhow, Result};
use rand::prelude::*;
use reqwest::{StatusCode, Url};
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::InputFile;
use tracing::info;

use crate::bot::command::PublicCommand;
use crate::bot::handlers::cmd_best_keyboard;
use crate::bot::handlers::utils::{cmd_best_text, cmd_challenge_keyboard, url_of};
use crate::bot::utils::ChallengeLocker;
use crate::bot::Bot;
use crate::config::Config;
use crate::database::{ChallengeView, GalleryEntity, MessageEntity};
use crate::ehentai::{EhGalleryUrl, GalleryInfo};
use crate::reply_to;
use crate::tags::EhTagTransDB;
use crate::uploader::ExloliUploader;

pub fn public_command_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription>
{
    teloxide::filter_command::<PublicCommand, _>()
        .branch(case![PublicCommand::Query(gallery)].endpoint(cmd_query))
        .branch(case![PublicCommand::Ping].endpoint(cmd_ping))
        .branch(case![PublicCommand::Update(url)].endpoint(cmd_update))
        .branch(case![PublicCommand::Best(from, to)].endpoint(cmd_best))
        .branch(case![PublicCommand::Challenge].endpoint(cmd_challenge))
}

async fn cmd_challenge(
    bot: Bot,
    msg: Message,
    trans: EhTagTransDB,
    locker: ChallengeLocker,
) -> Result<()> {
    info!("{}: /challenge", msg.from().unwrap().id);
    let challenge = ChallengeView::get_random().await?;
    let answer = challenge.choose(&mut thread_rng()).unwrap();
    let id = locker.add_challenge(answer.id, answer.page, answer.artist.clone());
    let keyboard = cmd_challenge_keyboard(id, &challenge, &trans);
    bot.send_photo(
        msg.chat.id,
        InputFile::url(format!("https://telegra.ph{}", answer.url).parse()?),
    )
    .caption("上述图片来自下列哪位作者的本子？")
    .reply_markup(keyboard)
    .reply_to_message_id(msg.id)
    .await?;
    Ok(())
}

async fn cmd_best(bot: Bot, msg: Message, (end, start): (u16, u16), cfg: Config) -> Result<()> {
    info!("{}: /best {} {}", msg.from().unwrap().id, end, start);
    let text = cmd_best_text(start as i32, end as i32, 0, cfg.telegram.channel_id).await?;
    let keyboard = cmd_best_keyboard(start as i32, end as i32, 0);
    reply_to!(bot, msg, text).reply_markup(keyboard).await?;
    Ok(())
}

async fn cmd_update(bot: Bot, msg: Message, uploader: ExloliUploader, url: Url) -> Result<()> {
    info!("{}: /update {}", msg.from().unwrap().id, url);
    let reply = reply_to!(bot, msg, "更新中……").await?;
    let msg_id = url
        .path_segments()
        .and_then(|p| p.last())
        .and_then(|id| id.parse::<i32>().ok())
        .ok_or(anyhow!("Invalid URL"))?;
    let msg_entity = MessageEntity::get(msg_id).await?.ok_or(anyhow!("Message not found"))?;
    let gl_entity =
        GalleryEntity::get(msg_entity.gallery_id).await?.ok_or(anyhow!("Gallery not found"))?;

    tokio::spawn(async move {
        // 文章被删了，需要重新发布文章
        if reqwest::get(&msg_entity.telegraph).await?.status() == StatusCode::NOT_FOUND {
            uploader.update_history_gallery_inner(&gl_entity).await?;
        }

        uploader.try_update(&gl_entity.url(), false).await?;
        bot.edit_message_text(msg.chat.id, reply.id, "更新完成").await?;
        Result::<()>::Ok(())
    });

    Ok(())
}

async fn cmd_ping(bot: Bot, msg: Message) -> Result<()> {
    info!("{}: /ping", msg.from().unwrap().id);
    reply_to!(bot, msg, "pong~").await?;
    Ok(())
}

async fn cmd_query(bot: Bot, msg: Message, cfg: Config, gallery: EhGalleryUrl) -> Result<()> {
    info!("{}: /query {}", msg.from().unwrap().id, gallery);
    match GalleryEntity::get(gallery.id()).await? {
        Some(gallery) => {
            let message = MessageEntity::get_by_gallery_id(gallery.id).await?.unwrap();
            let url = url_of(cfg.telegram.channel_id, message.id);
            reply_to!(bot, msg, url).await?;
        }
        None => {
            reply_to!(bot, msg, "未找到").await?;
        }
    }
    Ok(())
}
