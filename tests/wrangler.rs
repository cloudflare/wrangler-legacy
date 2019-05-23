use assert_cmd::prelude::*;

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn it_generates() {
    let mut wrangler = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    wrangler.arg("generate").assert();

    assert_eq!(Path::new("worker").exists(), true);
    assert_eq!(Path::new("worker/wrangler.toml").exists(), true);
    fs::remove_dir_all("worker").unwrap();
    assert_eq!(Path::new("worker").exists(), false);
}
