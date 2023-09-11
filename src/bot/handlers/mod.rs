mod callback_query;
mod command_admin;
mod command_public;
mod custom_poll;
mod join_request;
mod utils;

pub use callback_query::*;
pub use command_admin::*;
pub use command_public::*;
pub use custom_poll::*;
pub use join_request::*;
pub use utils::*;

#[macro_export]
macro_rules! reply_to {
    ($b:expr, $m:expr, $t:expr) => {
        $b.send_message($m.chat.id, $t).reply_to_message_id($m.id)
    };
}

#[macro_export]
macro_rules! try_with_reply {
    ($bot:expr, $msg:expr, $func:expr) => {
        let reply = reply_to!($bot, $msg, "执行中……").await?;
        match $func {
            Ok(_) => $bot.edit_message_text($msg.chat.id, reply.id, "执行成功").await?,
            Err(e) => $bot.edit_message_text($msg.chat.id, reply.id, format!("执行失败：{}", e)).await?,
        }
    };
}
