use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use crate::commands::validate_worker_name;
use crate::settings::toml::{Manifest, Site, TargetType};
use crate::{commands, install};

pub fn generate(
    name: &str,
    template: &str,
    target_type: Option<TargetType>,
    site: bool,
) -> Result<(), failure::Error> {
    validate_worker_name(name)?;

    log::info!("Generating a new worker project with name '{}'", name);
    run_generate(name, template)?;

    let config_path = PathBuf::from("./").join(&name);
    // TODO: this is tightly coupled to our site template. Need to remove once
    // we refine our generate logic.
    let generated_site = if site {
        Some(Site::new("./public"))
    } else {
        None
    };
    Manifest::generate(name.to_string(), target_type, &config_path, generated_site)?;

    Ok(())
}

pub fn run_generate(name: &str, template: &str) -> Result<(), failure::Error> {
    let tool_name = "cargo-generate";
    let binary_path = install::install(tool_name, "ashleygwilliams")?.binary(tool_name)?;

    let new_name = match generate_name(name) {
        Ok(val) => val,
        Err(_) => {
            log::debug!(
                "Failed to auto-increment name for a new worker project, using '{}'",
                name
            );
            String::from(name)
        }
    };

    let args = [
        "generate", "--git", template, "--name", &new_name, "--force",
    ];

    let command = command(binary_path, &args);
    let command_name = format!("{:?}", command);
    commands::run(command, &command_name)
}

fn generate_name(name: &str) -> Result<String, failure::Error> {
    let digits = detect_num(name);
    let mut new_name = String::from(name);
    let mut chars = new_name.chars().collect::<Vec<char>>();
    let split = chars.split_off(new_name.len() - digits);
    let bare_name = chars.into_iter().collect::<String>();

    let mut num = match split.into_iter().collect::<String>().parse::<usize>() {
        Ok(val) => Some(val),
        Err(_) => None,
    };

    let entry_names = read_current_dir()?;

    while entry_names.contains(&OsString::from(&new_name)) {
        num = num.map(|val| val + 1);
        new_name = construct_name(&bare_name, num);
        if num.is_none() {
            num = Some(1);
        }
    }
    Ok(new_name)
}

fn read_current_dir() -> Result<Vec<OsString>, failure::Error> {
    let current_dir = env::current_dir()?;
    let mut entry_names = Vec::new();

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        entry_names.push(entry.file_name());
    }
    Ok(entry_names)
}

fn construct_name(bare_name: &str, num: Option<usize>) -> String {
    let new_num = match num {
        Some(val) => val.to_string(),
        None => String::from("1"),
    };
    String::from(bare_name) + &new_num
}

fn detect_num(name: &str) -> usize {
    let digits = String::from(name).chars().rev().fold(0, |memo, item| {
        if item.is_digit(10) {
            memo + 1
        } else {
            return memo;
        }
    });
    digits
}

fn command(binary_path: PathBuf, args: &[&str]) -> Command {
    let mut c = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c.arg(binary_path);
        c
    } else {
        Command::new(binary_path)
    };

    c.args(args);
    c
}
