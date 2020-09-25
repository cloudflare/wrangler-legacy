use std::env;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::terminal::emoji;
use crate::terminal::message::{Message, StdOut};

const OUTPUT_DIR: &str = "dist";
const SRC_DIR: &str = "src";
const BUILD_COMMAND: &str = "npm run build";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    #[serde(rename = "build-command")]
    build_command: Option<String>,
    #[serde(rename = "output-dir")]
    output_dir: Option<PathBuf>,
    #[serde(rename = "src-dir")]
    src_dir: Option<PathBuf>,
}

impl Bundle {
    fn dir_or_default(dir: &Option<PathBuf>, default: &str, dirname: &str) -> PathBuf {
        match dir {
            Some(path) => path.to_owned(),
            None => {
                StdOut::warn(&format!(
                    "{} {} not specified, falling back to {}",
                    emoji::WARN,
                    dirname,
                    default
                ));
                PathBuf::from(default)
            }
        }
    }

    pub fn output_dir(&self) -> Result<PathBuf, std::io::Error> {
        let current_dir = env::current_dir()?;
        let output_dir = current_dir.join(Bundle::dir_or_default(
            &self.output_dir,
            OUTPUT_DIR,
            "output dir",
        ));
        Ok(output_dir)
    }

    pub fn src_dir(&self) -> Result<PathBuf, std::io::Error> {
        let current_dir = env::current_dir()?;
        let src_dir = current_dir.join(Bundle::dir_or_default(&self.src_dir, SRC_DIR, "src dir"));
        Ok(src_dir)
    }

    pub fn build_command(&self) -> Command {
        let args_string = match &self.build_command {
            Some(cmd) => cmd.to_owned(),
            None => {
                StdOut::warn(&format!(
                    "{} build command not specified, falling back to {}",
                    emoji::WARN,
                    BUILD_COMMAND
                ));
                BUILD_COMMAND.to_string()
            }
        };
        let args: Vec<&str> = args_string.split_whitespace().collect();

        let command = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.arg("/C").args(args.as_slice());
            c
        } else {
            let mut c = Command::new(args[0]);
            if args.len() > 1 {
                c.args(&args[1..]);
            }
            c
        };

        command
    }
}
