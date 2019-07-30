use assert_cmd::prelude::*;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::{Child, Command, Stdio};

#[test]
fn it_generates_the_config() {
    let fake_home_dir = env::current_dir()
        .expect("could not retrieve cwd")
        .join(".it_generates_the_config");
    let cmd = config_with_wrangler_home(fake_home_dir.to_str().unwrap());
    let mut stdin = cmd.stdin.unwrap();

    write!(stdin, "a\n").unwrap(); // email
    write!(stdin, "b\n").unwrap(); // api_key

    let mut buffer = "".to_string();
    let mut stdout = cmd.stdout.unwrap();
    stdout
        .read_to_string(&mut buffer)
        .expect("could not read output");
    assert!(buffer.contains("Enter email: \nEnter api key: \n Successfully configured."));

    let config_file = fake_home_dir.join("config").join("default.toml");

    let config = fs::read_to_string(&config_file)
        .expect(&format!("could not read config at {:?}", &config_file));
    assert_eq!(
        config,
        r#"email = "a"
api_key = "b"
"#
    );

    // check dir permissions (but not on windows)
    if !cfg!(target_os = "windows") {
        let mut command = Command::new("stat");
        command.arg("-c");
        command.arg("%a %n");
        command.arg(&config_file);
        let out = String::from_utf8(command.output().expect("could not stat file").stdout).unwrap();
        // stat format is: "mode file"
        assert!(out.starts_with("600"));
    }

    fs::remove_dir_all(&fake_home_dir).expect("could not delete dir");
}

fn config_with_wrangler_home(home_dir: &str) -> Child {
    let mut wrangler = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    wrangler
        .arg("config")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .env("WRANGLER_HOME", home_dir)
        .spawn()
        .unwrap()
}
