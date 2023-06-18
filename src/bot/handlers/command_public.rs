use anyhow::Result;
use teloxide::prelude::*;

use super::super::command::PublicCommand;
use crate::database::GalleryEntity;

pub async fn public_command_handler(bot: Bot, msg: Message, cmd: PublicCommand) -> Result<()> {
    Ok(())
}
