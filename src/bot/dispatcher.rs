use teloxide::prelude::*;
use teloxide::types::ChatMemberKind;

use super::command::*;
use super::filter::*;
use super::handlers::*;

pub async fn dispatcher(bot: Bot) {
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .filter_command::<AdminCommand>()
                        .chain(filter_admin_msg())
                        .endpoint(admin_command_handler),
                )
                .branch(
                    dptree::entry()
                        .filter_command::<PublicCommand>()
                        .endpoint(public_command_handler),
                ),
        )
        .branch(Update::filter_poll().endpoint(poll_handler))
        .branch(Update::filter_inline_query().endpoint(inline_query_handler))
        .branch(Update::filter_callback_query().endpoint(callback_query_handler));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
