use assert_cmd::prelude::*;

use std::process::Command;

#[test]
fn it_has_a_help_flag() {
    let mut wrangler = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    wrangler.arg("--help").assert();
}
