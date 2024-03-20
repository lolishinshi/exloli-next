use std::time::Duration;

use teloxide::prelude::*;

use super::filter::{filter_callbackdata, filter_channel_msg};
use super::handlers::*;
use super::utils::{ChallengeLocker, ChallengeProvider, RateLimiter};
use super::Bot;
use crate::bot::scheduler::Scheduler;
use crate::config::Config;
use crate::tags::EhTagTransDB;
use crate::uploader::ExloliUploader;

pub async fn start_dispatcher(
    config: Config,
    ehentai: ExloliUploader,
    bot: Bot,
    trans: EhTagTransDB,
) {
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(admin_command_handler())
                .branch(public_command_handler(config.clone()))
                .branch(filter_channel_msg().endpoint(custom_pool_sender)),
        )
        .branch(
            Update::filter_callback_query()
                .chain(filter_callbackdata())
                .chain(callback_query_handler()),
        )
        .branch(Update::filter_chat_join_request().endpoint(join_request_handler));

    // 限制每 60 秒只能进行 10 次操作
    let rate_limiter = RateLimiter::new(Duration::from_secs(60), 10);

    let challenge_locker = ChallengeLocker::new();

    let challenge_provider = ChallengeProvider::new();

    let scheduler = Scheduler::new(bot.clone());

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            ehentai,
            config,
            rate_limiter,
            trans,
            challenge_locker,
            scheduler,
            challenge_provider
        ])
        // NOTE: 默认情况下，同一个分组内的消息是串行处理，不同分组内的消息是并行处理
        // 此处使用空的分组函数，这样所有消息都会并行处理
        .distribution_function(|_| None::<()>)
        .build()
        .dispatch()
        .await;
}
