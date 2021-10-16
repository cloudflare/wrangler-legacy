extern crate base64;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};

use crate::kv::bulk::put;
use crate::kv::bulk::BATCH_KEY_MAX;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdErr};
pub fn run(target: &Target, user: &GlobalUser, namespace_id: &str, filename: &Path) -> Result<()> {
    let pairs: Vec<KeyValuePair> = match &metadata(filename) {
        Ok(file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            let data_vec = serde_json::from_str(&data);
            match data_vec {
                Ok(data_vec) => Ok(data_vec),
                Err(_) => Err(anyhow!("Failed to decode JSON. Please make sure to follow the format, [{{\"key\": \"test_key\", \"value\": \"test_value\"}}, ...]"))
            }
        }
        Ok(_) => Err(anyhow!(
            "{} should be a JSON file, but is not",
            filename.display()
        )),
        Err(e) => Err(anyhow!("{}", e)),
    }?;

    let len = pairs.len();

    StdErr::working(&format!("uploading {} key value pairs", len));
    let progress_bar = if len > BATCH_KEY_MAX {
        let pb = ProgressBar::new(len as u64);
        pb.set_style(ProgressStyle::default_bar().template("{wide_bar} {pos}/{len}\n{msg}"));
        Some(pb)
    } else {
        None
    };

    put(target, user, namespace_id, pairs, &progress_bar)?;

    if let Some(pb) = &progress_bar {
        pb.finish_with_message(&format!("uploaded {} key value pairs", len));
    }

    StdErr::success("Success");
    Ok(())
}
