use crate::terminal::message::{Message, StdOut};
use std::process::{Command, Stdio};

use anyhow::Result;

pub fn open_browser(url: &str, log_url: bool) -> Result<()> {
    if cfg!(target_os = "windows") {
        let url_escaped = url.replace("&", "^&");
        let windows_cmd = format!("start {}", url_escaped);
        Command::new("cmd")
            .args(&["/C", &windows_cmd])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    } else if cfg!(target_os = "linux") {
        let linux_cmd = format!(r#"xdg-open "{}""#, url);
        Command::new("sh")
            .arg("-c")
            .arg(&linux_cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    } else {
        let mac_cmd = format!(r#"open "{}""#, url);
        Command::new("sh")
            .arg("-c")
            .arg(&mac_cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    };

    let msg = match log_url {
        true => format!("Opened a link in your default browser: {}", url),
        false => "Opened a link in your default browser.".to_string(),
    };

    StdOut::info(&msg);
    Ok(())
}
