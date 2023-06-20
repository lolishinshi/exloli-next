use anyhow::{anyhow, Result};
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
use crate::manager::uploader::ExloliUploader;
use crate::reply_to;

pub fn admin_command_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription>
{
    teloxide::filter_command::<AdminCommand, _>()
        .chain(filter_admin_msg())
        .branch(case![AdminCommand::Upload(gallery)].endpoint(cmd_upload))
        .branch(case![AdminCommand::Delete].endpoint(cmd_delete))
        .branch(case![AdminCommand::Erase].endpoint(cmd_erase))
}

async fn cmd_upload(
    bot: Bot,
    msg: Message,
    uploader: ExloliUploader,
    gallery: EhGalleryUrl,
) -> Result<()> {
    let reply = reply_to!(bot, msg, "上传中……").await?;
    uploader.check_and_upload(&gallery).await?;
    bot.edit_message_text(msg.chat.id, reply.id, "上传完成").await?;
    Ok(())
}

async fn cmd_delete(bot: Bot, msg: Message) -> Result<()> {
    let reply_to = msg.reply_to_message().ok_or(anyhow!("No reply message"))?;
    let channel = reply_to.forward_from_chat().ok_or(anyhow!("No forward from chat"))?;
    let msg_entity = MessageEntity::get(reply_to.id.0).await?.unwrap();
    bot.delete_message(channel.id, MessageId(msg_entity.id)).await?;
    GalleryEntity::update_deleted(msg_entity.gallery_id, true).await?;
    Ok(())
}

async fn cmd_erase(bot: Bot, msg: Message) -> Result<()> {
    let reply_to = msg.reply_to_message().ok_or(anyhow!("No reply message"))?;
    let channel = reply_to.forward_from_chat().ok_or(anyhow!("No forward from chat"))?;
    let msg_entity = MessageEntity::get(reply_to.id.0).await?.unwrap();
    bot.delete_message(reply_to.chat.id, reply_to.id).await?;
    bot.delete_message(channel.id, MessageId(msg_entity.id)).await?;
    GalleryEntity::delete(msg_entity.gallery_id).await?;
    Ok(())
}
