use config::log_file;
use config::log_level;
use lazy_static::lazy_static;
use slog::Drain;
use std::fs::OpenOptions;

pub mod client;
pub mod cmd;
pub mod config;
pub mod llama;
pub mod session;
pub mod utils;

lazy_static! {
    pub static ref LOGGER: slog::Logger = slog::Logger::root(
        slog_term::FullFormat::new(slog_term::PlainSyncDecorator::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(log_file())
                .unwrap()
        ))
        .use_custom_timestamp(utils::timestamp_local)
        .build()
        .filter_level(slog::Level::from_usize(log_level()).unwrap())
        .fuse(),
        slog::o!()
    );
}

/// Error returned by most functions.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type.
///
/// This is defined as a convenience.
pub type Result<T> = anyhow::Result<T, Error>;

pub const USER_CHATTING_NAME: &str = "Userc33dc3a";
pub const USER_CHATTING_NAME_SHORT: &str = "Userc33dc3";
