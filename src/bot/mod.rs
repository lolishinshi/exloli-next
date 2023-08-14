mod command;
mod dispatcher;
mod filter;
mod handlers;
mod scheduler;
mod utils;

pub use dispatcher::start_dispatcher;
use teloxide::adaptors::{CacheMe, DefaultParseMode, Throttle};

pub type Bot = CacheMe<DefaultParseMode<Throttle<teloxide::Bot>>>;
