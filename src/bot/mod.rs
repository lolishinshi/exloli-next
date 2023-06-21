mod command;
mod dispatcher;
mod filter;
mod handlers;
mod utils;

pub use dispatcher::start_dispatcher;
use teloxide::adaptors::{CacheMe, DefaultParseMode, Throttle};

pub type Bot = Throttle<CacheMe<DefaultParseMode<teloxide::Bot>>>;
