#![cfg_attr(feature = "strict", deny(warnings))]

#[macro_use]
extern crate text_io;

mod build;
pub mod preview;
pub mod util;
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
