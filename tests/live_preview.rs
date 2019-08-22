use assert_cmd::prelude::*;
use fs_extra::dir::{copy, CopyOptions};
use std::env;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

const WATCH_TIMEOUT: Duration = Duration::from_secs(15);
const WATCH_TIMEOUT_RUST: Duration = Duration::from_secs(60);

macro_rules! settings {
    ( $f:expr, $x:expr ) => {
        let file_path = fixture_path($f).join("wrangler.toml");
        let mut file = File::create(file_path).unwrap();
        let content = format!(
            r#"
            name = "test"
            zone_id = ""
            account_id = ""
            {}
        "#,
            $x
        );
        file.write_all(content.as_bytes()).unwrap();
    };
}

macro_rules! append_file {
    ( $f:expr, $f2:expr, $x:expr ) => {
        println!("appending to {}", $f2);
        let file_path = fixture_path($f).join($f2);
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(file_path)
            .unwrap();
        let content = format!($x);
        file.write_all(content.as_bytes()).unwrap();
    };
}

#[test]
fn it_can_watch_js_project() {
    let fixture = "simple_js";
    create_temporary_copy(fixture);
    settings! {fixture, r#"
        type = "javascript"
    "#};
    let rx = live_watch(fixture);
    let handle = thread::spawn(move || rx.recv_timeout(WATCH_TIMEOUT).unwrap());
    println!("spawning append thread");
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(5));
        append_file! { fixture, "index.js", r#"console.log("hello")"# }
    });
    handle.join().unwrap();
    cleanup(fixture);
}

#[test]
fn it_can_watch_webpack_project() {
    let fixture = "webpack_simple_js";
    create_temporary_copy(fixture);
    settings! {fixture, r#"
        type = "webpack"
    "#};
    let rx = live_watch(fixture);
    let handle = thread::spawn(move || rx.recv_timeout(WATCH_TIMEOUT).unwrap());
    println!("spawning append thread");
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(5));
        append_file! { fixture, "index.js", r#"console.log("hello")"# };
    });
    handle.join().unwrap();
    cleanup(fixture);
}

#[test]
fn it_can_watch_rust_project() {
    let fixture = "simple_rust";
    create_temporary_copy(fixture);
    settings! {fixture, r#"
        type = "rust"
    "#};
    let rx = live_watch(fixture);
    let handle = thread::spawn(move || rx.recv_timeout(WATCH_TIMEOUT_RUST).unwrap());
    println!("spawning append thread");
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(5));
        append_file! { fixture, "worker/worker.js", r#"console.log("hello")"# };
    });
    handle.join().unwrap();
    cleanup(fixture);
}

fn live_watch(fixture: &str) -> Receiver<()> {
    use wrangler::commands;
    use wrangler::settings::project::Project;
    let (tx, rx) = mpsc::channel();

    let dir = fixture_path(fixture);
    println!("{:#?}", dir);

    let project = Project::new(&dir).unwrap();

    commands::watch_and_build(&project, &dir, Some(tx)).unwrap();
    rx
}

fn cleanup(fixture: &str) {
    let path = fixture_path(fixture);
    assert!(path.exists(), format!("{:?} does not exist", path));

    // Workaround https://github.com/rust-lang/rust/issues/29497
    if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.arg("rmdir");
        command.arg("/s");
        command.arg(&path);
    } else {
        fs::remove_dir_all(&path).unwrap();
    }
}

fn fixture_path(fixture: &str) -> PathBuf {
    let mut dest = env::temp_dir();
    dest.push(fixture);
    dest
}

fn create_temporary_copy(fixture: &str) {
    let current_dir = env::current_dir().unwrap();
    let src = Path::new(&current_dir).join("tests/fixtures").join(fixture);

    let dest = env::temp_dir();

    if dest.join(fixture).exists() {
        cleanup(fixture);
    }

    fs::create_dir_all(dest.clone()).unwrap();
    let mut options = CopyOptions::new();
    options.overwrite = true;
    copy(src, dest, &options).unwrap();
}
