use anyhow::{anyhow, Result};
use reqwest::{StatusCode, Url};
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use tracing::instrument;

use crate::bot::command::PublicCommand;
use crate::bot::handlers::cmd_best_keyboard;
use crate::bot::handlers::utils::{cmd_best_text, url_of};
use crate::bot::Bot;
use crate::config::Config;
use crate::database::{GalleryEntity, MessageEntity};
use crate::ehentai::{EhGalleryUrl, GalleryInfo};
use crate::reply_to;
use crate::uploader::ExloliUploader;

pub fn public_command_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription>
{
    teloxide::filter_command::<PublicCommand, _>()
        .branch(case![PublicCommand::Query(gallery)].endpoint(cmd_query))
        .branch(case![PublicCommand::Ping].endpoint(cmd_ping))
        .branch(case![PublicCommand::Update(url)].endpoint(cmd_update))
        .branch(case![PublicCommand::Best(from, to)].endpoint(cmd_best))
}

async fn cmd_best(bot: Bot, msg: Message, (end, start): (u16, u16), cfg: Config) -> Result<()> {
    let text = cmd_best_text(start as i32, end as i32, 0, cfg.telegram.channel_id).await?;
    let keyboard = cmd_best_keyboard(start as i32, end as i32, 0);
    reply_to!(bot, msg, text).reply_markup(keyboard).await?;
    Ok(())
}

async fn cmd_update(bot: Bot, msg: Message, uploader: ExloliUploader, url: Url) -> Result<()> {
    let reply = reply_to!(bot, msg, "更新中……").await?;
    let msg_id = url
        .path_segments()
        .and_then(|p| p.last())
        .and_then(|id| id.parse::<i32>().ok())
        .ok_or(anyhow!("Invalid URL"))?;
    let msg_entity = MessageEntity::get(msg_id).await?.ok_or(anyhow!("Message not found"))?;
    let gl_entity =
        GalleryEntity::get(msg_entity.gallery_id).await?.ok_or(anyhow!("Gallery not found"))?;

    // 文章被删了，需要重新发布文章
    if reqwest::get(&msg_entity.telegraph).await?.status() == StatusCode::NOT_FOUND {
        uploader.republish(&gl_entity, &msg_entity).await?;
    }

    uploader.try_update(&gl_entity.url()).await?;
    bot.edit_message_text(msg.chat.id, reply.id, "更新完成").await?;
    Ok(())
}

#[instrument(skip(bot, msg))]
async fn cmd_ping(bot: Bot, msg: Message) -> Result<()> {
    reply_to!(bot, msg, "pong~").await?;
    Ok(())
}

#[instrument(skip(bot, msg, cfg))]
async fn cmd_query(bot: Bot, msg: Message, cfg: Config, gallery: EhGalleryUrl) -> Result<()> {
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
