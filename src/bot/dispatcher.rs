use std::time::Duration;

use teloxide::prelude::*;

use super::filter::{filter_callbackdata, filter_channel_msg};
use super::handlers::*;
use super::utils::RateLimiter;
use super::Bot;
use crate::config::Config;
use crate::uploader::ExloliUploader;

pub async fn start_dispatcher(config: Config, ehentai: ExloliUploader, bot: Bot) {
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(admin_command_handler())
                .branch(public_command_handler())
                .branch(filter_channel_msg().endpoint(custom_pool_sender)),
        )
        .branch(
            Update::filter_callback_query()
                .chain(filter_callbackdata())
                .chain(callback_query_handler()),
        );

    // 限制每 60 秒只能进行 10 次操作
    let rate_limiter = RateLimiter::new(Duration::from_secs(60), 10);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![ehentai, config, rate_limiter])
        .build()
        .dispatch()
        .await;
}
