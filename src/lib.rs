#![cfg_attr(feature = "strict", deny(warnings))]

#[macro_use]
extern crate text_io;

mod build;
pub mod preview;
pub use build::build;
pub mod commands;
pub mod deploy;
pub mod http;
pub mod install;
pub mod installer;
pub mod kv;
pub mod login;
pub mod settings;
pub mod sites;
pub mod tail;
pub mod terminal;
pub mod upload;
pub mod version;
pub mod watch;
pub mod wranglerjs;

pub mod fixtures;
