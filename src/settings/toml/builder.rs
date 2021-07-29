use std::env;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::upload::form::ModuleType;

const WATCH_DIR: &str = "src";
const UPLOAD_DIR: &str = "dist";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Builder {
    pub command: Option<String>,
    #[serde(default = "project_root")]
    pub cwd: PathBuf,
    #[serde(default = "watch_dir")]
    pub watch_dir: PathBuf,
    pub upload: UploadFormat,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "format")]
#[serde(deny_unknown_fields)]
pub enum UploadFormat {
    #[serde(rename = "service-worker")]
    ServiceWorker {},
    #[serde(rename = "modules")]
    Modules {
        main: String, // String since this is a module name, not a path.
        #[serde(default = "upload_dir")]
        dir: PathBuf,
        rules: Option<Vec<ModuleRule>>,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleRule {
    pub globs: Vec<String>,
    #[serde(rename = "type")]
    pub module_type: ModuleType,
    #[serde(default)] // false is default
    pub fallthrough: bool,
}

fn project_root() -> PathBuf {
    env::current_dir().unwrap()
}

fn watch_dir() -> PathBuf {
    project_root().join(WATCH_DIR)
}

fn upload_dir() -> PathBuf {
    std::env::current_dir().unwrap().join(UPLOAD_DIR)
}

impl Builder {
    pub fn verify_watch_dir(&self) -> Result<()> {
        let watch_canonical = match self.watch_dir.canonicalize() {
            Ok(path) => path,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound) => anyhow::bail!(
                "Your provided watch_dir {} does not exist.",
                self.watch_dir.display()
            ),
            Err(e) => anyhow::bail!(
                "Error encountered when verifying watch_dir: {}, provided path: {}",
                e,
                self.watch_dir.display()
            ),
        };
        let root_canonical = project_root().canonicalize()?;
        if watch_canonical == root_canonical {
            anyhow::bail!("Wrangler doesn't support using the project root as the watch_dir.");
        }
        if !self.watch_dir.is_dir() {
            anyhow::bail!(
                "A path was provided for watch_dir that is not a directory: {}",
                self.watch_dir.display()
            );
        }
        Ok(())
    }

    pub fn verify_upload_dir(&self) -> Result<()> {
        let dir = match &self.upload {
            UploadFormat::Modules { dir, .. } => dir,
            UploadFormat::ServiceWorker {} => return Ok(()),
        };

        let upload_canonical = match dir.canonicalize() {
            Ok(path) => path,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound) => {
                anyhow::bail!("Your provided upload_dir {} does not exist.", dir.display())
            }
            Err(e) => anyhow::bail!(
                "Error encountered when verifying upload_dir: {}, provided path: {}",
                e,
                dir.display()
            ),
        };
        let root_canonical = project_root().canonicalize()?;
        if upload_canonical == root_canonical {
            anyhow::bail!("Wrangler doesn't support using the project root as the upload_dir.");
        }
        if !dir.is_dir() {
            anyhow::bail!(
                "A path was provided for upload_dir that is not a directory: {}",
                dir.display()
            );
        }
        Ok(())
    }

    pub fn build_command(&self) -> Option<(&str, Command)> {
        match &self.command {
            Some(cmd) => {
                let mut c = if cfg!(target_os = "windows") {
                    let args: Vec<&str> = cmd.split_whitespace().collect();
                    let mut c = Command::new("cmd");
                    c.arg("/C");
                    c.args(args.as_slice());
                    c
                } else {
                    let mut c = Command::new("sh");
                    c.arg("-c");
                    c.arg(cmd);
                    c
                };

                c.current_dir(&self.cwd);

                Some((cmd, c))
            }
            None => None,
        }
    }
}
