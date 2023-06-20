mod callback_query;
mod command_admin;
mod command_public;
mod utils;

pub use callback_query::*;
pub use command_admin::*;
pub use command_public::*;

#[macro_export]
macro_rules! reply_to {
    ($b:expr, $m:expr, $t:expr) => {
        $b.send_message($m.chat.id, $t).reply_to_message_id($m.id)
    };
}
