use anyhow::Result;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::{MessageId, Recipient};
use tracing::instrument;

use super::super::command::PublicCommand;
use crate::config::Config;
use crate::database::{GalleryEntity, MessageEntity};
use crate::ehentai::EhGalleryUrl;
use crate::reply_to;

pub fn public_command_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription>
{
    teloxide::filter_command::<PublicCommand, _>()
        .branch(case![PublicCommand::Query(gallery)].endpoint(cmd_query))
}

#[instrument(skip(bot, msg, cfg))]
async fn cmd_query(gallery: EhGalleryUrl, bot: Bot, msg: Message, cfg: Config) -> Result<()> {
    match GalleryEntity::get(gallery.id()).await? {
        Some(gallery) => {
            let message = MessageEntity::get_by_gallery_id(gallery.id).await?.unwrap();
            let url = match cfg.telegram.channel_id {
                Recipient::Id(chatid) => Message::url_of(chatid, None, MessageId(message.id)),
                Recipient::ChannelUsername(username) => {
                    Message::url_of(ChatId(0), Some(&username[1..]), MessageId(message.id))
                }
            };
            reply_to!(bot, msg, format!("{}", url.unwrap())).await?;
        }
        None => {
            reply_to!(bot, msg, "未找到").await?;
        }
    }
    Ok(())
}
