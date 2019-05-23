use assert_cmd::prelude::*;

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn it_generates() {
    let mut wrangler = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    wrangler.arg("generate").assert();

    assert_eq!(Path::new("wasm-worker").exists(), true);
    assert_eq!(Path::new("wasm-worker/wrangler.toml").exists(), true);
    fs::remove_dir_all("wasm-worker").unwrap();
    assert_eq!(Path::new("wasm-worker").exists(), false);
}
