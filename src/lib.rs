#![cfg_attr(feature = "strict", deny(warnings))]

#[macro_use]
extern crate text_io;

mod build;
pub mod cli;
pub mod preview;
pub use build::build_target;
pub mod commands;
pub mod deploy;
pub mod http;
pub mod install;
pub mod installer;
pub mod kv;
pub mod login;
pub mod reporter;
pub mod settings;
pub mod sites;
pub mod tail;
pub mod terminal;
pub mod upload;
pub mod version;
pub mod watch;
pub mod wranglerjs;

pub mod fixtures;

const TEMP_NOTICE_ES_MODULES_DO_BETA: &str = "Your account does not have permission to do this! While Durable Objects are in Beta, the modules format is limited to accounts which have opted-in to the Beta. You may do so by following the instructions here: https://developers.cloudflare.com/workers/learning/using-durable-objects";
