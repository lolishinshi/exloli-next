use anyhow::{anyhow, Context, Result};
use chrono::{Duration, Utc};
use rand::prelude::*;
use reqwest::Url;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::{ChatMemberKind, InputFile};
use tracing::info;

use crate::bot::command::PublicCommand;
use crate::bot::filter::{filter_member, filter_private_chat};
use crate::bot::handlers::{
    cmd_best_keyboard, cmd_best_text, cmd_challenge_keyboard, gallery_preview_url,
};
use crate::bot::utils::ChallengeLocker;
use crate::bot::Bot;
use crate::config::Config;
use crate::database::{
    ChallengeView, GalleryEntity, InviteLink, MessageEntity, PageEntity, PollEntity,
};
use crate::ehentai::{EhGalleryUrl, GalleryInfo};
use crate::reply_to;
use crate::tags::EhTagTransDB;
use crate::uploader::ExloliUploader;

pub fn public_command_handler(
    config: Config,
) -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription> {
    teloxide::filter_command::<PublicCommand, _>()
        .branch(case![PublicCommand::Query(gallery)].endpoint(cmd_query))
        .branch(case![PublicCommand::Ping].endpoint(cmd_ping))
        .branch(case![PublicCommand::Update(url)].endpoint(cmd_update))
        .branch(case![PublicCommand::Best(from, to)].endpoint(cmd_best))
        .branch(case![PublicCommand::Challenge].endpoint(cmd_challenge))
        .branch(case![PublicCommand::Upload(gallery)].endpoint(cmd_upload))
        .branch(case![PublicCommand::Invite].endpoint(cmd_invite))
}

async fn cmd_invite(bot: Bot, msg: Message, cfg: Config) -> Result<()> {
    let user = msg.from().unwrap().id;

    info!("{}: /invite", user);

    if !msg.chat.is_private() {
        return Ok(());
    }

    if matches!(
        bot.get_chat_member(cfg.telegram.auth_group_id, user).await?.kind,
        ChatMemberKind::Restricted(_) | ChatMemberKind::Banned(_) | ChatMemberKind::Left
    ) {
        reply_to!(bot, msg, "您尚未加入讨论组").await?;
        return Ok(());
    }
    if !matches!(
        bot.get_chat_member(cfg.telegram.channel_id.clone(), user).await?.kind,
        ChatMemberKind::Left
    ) {
        reply_to!(bot, msg, "您已经加入，或者被限制加入群组").await?;
        return Ok(());
    }

    if let Some(link) = InviteLink::get(user.0 as i64).await? {
        reply_to!(bot, msg, format!("你的邀请链接是：{}", link.link)).await?;
    } else {
        let link = bot
            .create_chat_invite_link(cfg.telegram.channel_id)
            .member_limit(1)
            .expire_date(Utc::now() + Duration::hours(1))
            .await?;
        InviteLink::create(user.0 as i64, &link.invite_link).await?;
        reply_to!(bot, msg, format!("邀请链接：{}\n有效次数：1\n有效期：1 小时", link.invite_link))
            .await?;
    }
    Ok(())
}

async fn cmd_upload(
    bot: Bot,
    msg: Message,
    uploader: ExloliUploader,
    gallery: EhGalleryUrl,
) -> Result<()> {
    info!("{}: /upload {}", msg.from().unwrap().id, gallery);
    if GalleryEntity::get(gallery.id()).await?.is_none() {
        reply_to!(bot, msg, "非管理员只能上传存在上传记录的画廊").await?;
    } else {
        let reply = reply_to!(bot, msg, "上传中……").await?;
        tokio::spawn(async move {
            uploader.try_upload(&gallery, true).await?;
            bot.edit_message_text(msg.chat.id, reply.id, "上传完成").await?;
            Result::<()>::Ok(())
        });
    }
    Ok(())
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
    reply_to!(bot, msg, text).reply_markup(keyboard).disable_web_page_preview(true).await?;
    Ok(())
}

async fn cmd_update(bot: Bot, msg: Message, uploader: ExloliUploader, url: String) -> Result<()> {
    info!("{}: /update {}", msg.from().unwrap().id, url);
    let reply = reply_to!(bot, msg, "更新中……").await?;
    let msg_id = if url.is_empty() {
        msg.reply_to_message()
            .and_then(|msg| msg.forward_from_message_id())
            .ok_or(anyhow!("Invalid URL"))?
    } else {
        Url::parse(&url)?
            .path_segments()
            .and_then(|p| p.last())
            .and_then(|id| id.parse::<i32>().ok())
            .ok_or(anyhow!("Invalid URL"))?
    };
    let msg_entity = MessageEntity::get(msg_id).await?.ok_or(anyhow!("Message not found"))?;
    let gl_entity =
        GalleryEntity::get(msg_entity.gallery_id).await?.ok_or(anyhow!("Gallery not found"))?;

    tokio::spawn(async move {
        // 如果检测到页面数量和实际页面数量不一致，需要重新发布文章
        let force_republish = gl_entity.pages != PageEntity::count(gl_entity.id).await?;
        // 调用 update_history_gallery_inner 来检测是否是缺页的画廊（包括旧画廊和异常画廊）
        // 顺便还会把失效画廊重新上传
        uploader.rescan_gallery(&gl_entity).await?;
        if force_republish {
            uploader.republish(&gl_entity, &msg_entity).await?;
        }
        // 最后看一下有没有 tag 或者标题需要更新
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
            let poll = PollEntity::get_by_gallery(gallery.id).await?.context("找不到投票")?;
            let preview = gallery_preview_url(cfg.telegram.channel_id, gallery.id).await?;
            let url = gallery.url().url();
            reply_to!(
                bot,
                msg,
                format!("消息：{preview}\n地址：{url}\n评分：{:.2}", poll.score * 100.)
            )
            .await?;
        }
        None => {
            reply_to!(bot, msg, "未找到").await?;
        }
    }
    Ok(())
}
