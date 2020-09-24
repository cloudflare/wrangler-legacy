use std::env;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

const OUTPUT_DIR: &str = "dist";
const SRC_DIR: &str = "src";

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
    fn dir_or_default(dir: &Option<PathBuf>, default: &str) -> PathBuf {
        match dir {
            Some(path) => path.to_owned(),
            None => PathBuf::from(default),
        }
    }

    pub fn output_dir(&self) -> Result<PathBuf, std::io::Error> {
        let current_dir = env::current_dir()?;
        let output_dir = current_dir.join(Bundle::dir_or_default(&self.output_dir, OUTPUT_DIR));
        Ok(output_dir)
    }

    pub fn src_dir(&self) -> Result<PathBuf, std::io::Error> {
        let current_dir = env::current_dir()?;
        let src_dir = current_dir.join(Bundle::dir_or_default(&self.src_dir, SRC_DIR));
        Ok(src_dir)
    }

    pub fn build_command(&self) -> Command {
        let args_string = match &self.build_command {
            Some(cmd) => cmd.to_owned(),
            None => "npm run build".to_string(),
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
