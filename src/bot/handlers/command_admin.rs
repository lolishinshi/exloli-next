use anyhow::{Context, Result};
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::MessageId;
use tracing::info;

use crate::bot::command::AdminCommand;
use crate::bot::filter::filter_admin_msg;
use crate::bot::Bot;
use crate::database::{GalleryEntity, MessageEntity};
use crate::ehentai::EhGalleryUrl;
use crate::uploader::ExloliUploader;
use crate::{reply_to, try_with_reply};

pub fn admin_command_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription>
{
    teloxide::filter_command::<AdminCommand, _>()
        .chain(filter_admin_msg())
        .branch(case![AdminCommand::Upload(gallery)].endpoint(cmd_upload))
        .branch(case![AdminCommand::Delete].endpoint(cmd_delete))
        .branch(case![AdminCommand::Erase].endpoint(cmd_delete))
        .branch(case![AdminCommand::ReCheck].endpoint(cmd_recheck))
        .branch(case![AdminCommand::ReUpload].endpoint(cmd_reupload))
}

// TODO: 该功能需要移除
async fn cmd_reupload(bot: Bot, msg: Message, uploader: ExloliUploader) -> Result<()> {
    info!("{}: /reupload", msg.from().unwrap().id);
    try_with_reply!(bot, msg, uploader.reupload(vec![]).await);
    Ok(())
}

async fn cmd_recheck(bot: Bot, msg: Message, uploader: ExloliUploader) -> Result<()> {
    info!("{}: /recheck", msg.from().unwrap().id);
    try_with_reply!(bot, msg, uploader.recheck(vec![]).await);
    Ok(())
}

async fn cmd_upload(
    bot: Bot,
    msg: Message,
    uploader: ExloliUploader,
    gallery: EhGalleryUrl,
) -> Result<()> {
    info!("{}: /upload {}", msg.from().unwrap().id, gallery);
    try_with_reply!(bot, msg, uploader.try_upload(&gallery, false).await);
    Ok(())
}

async fn cmd_delete(bot: Bot, msg: Message, command: AdminCommand) -> Result<()> {
    info!("{}: /delete", msg.from().unwrap().id);
    let reply_to = msg.reply_to_message().context("没有回复消息")?;

    let channel = reply_to.forward_from_chat().context("该消息没有回复画廊")?;
    let channel_msg = reply_to.forward_from_message_id().context("获取转发来源失败")?;

    let msg_entity = MessageEntity::get(channel_msg).await?.unwrap();

    bot.delete_message(reply_to.chat.id, reply_to.id).await?;
    bot.delete_message(channel.id, MessageId(msg_entity.id)).await?;

    if matches!(command, AdminCommand::Delete) {
        GalleryEntity::update_deleted(msg_entity.gallery_id, true).await?;
    } else {
        GalleryEntity::delete(msg_entity.gallery_id).await?;
        MessageEntity::delete(channel_msg).await?;
    }

    Ok(())
}
