extern crate base64;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

use crate::commands::kv;
use crate::kv::bulk::delete;
use crate::kv::bulk::BATCH_KEY_MAX;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::interactive;
use crate::terminal::message::{Message, StdOut};
pub fn run(target: &Target, user: &GlobalUser, namespace_id: &str, filename: &Path) -> Result<()> {
    kv::validate_target(target)?;

    match interactive::confirm(&format!(
        "Are you sure you want to delete all keys in {}?",
        filename.display()
    )) {
        Ok(true) => (),
        Ok(false) => {
            StdOut::info(&format!("Not deleting keys in {}", filename.display()));
            return Ok(());
        }
        Err(e) => anyhow::bail!(e),
    }

    let keys: Vec<String> = match &metadata(filename) {
        Ok(file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            let keys_vec: Result<Vec<KeyValuePair>, serde_json::Error> =
                serde_json::from_str(&data);
            match keys_vec {
                Ok(keys_vec) => keys_vec.iter().map(|kv| { kv.key.to_owned() }).collect(),
                Err(_) => anyhow::bail!("Failed to decode JSON. Please make sure to follow the format, [{\"key\": \"test_key\", \"value\": \"test_value\"}, ...]")
            }
        }
        Ok(_) => anyhow::bail!("{} should be a JSON file, but is not", filename.display()),
        Err(e) => anyhow::bail!("{}", e),
    };

    let len = keys.len();

    StdOut::working(&format!("deleting {} key value pairs", len));

    let progress_bar = if len > BATCH_KEY_MAX {
        let pb = ProgressBar::new(len as u64);
        pb.set_style(ProgressStyle::default_bar().template("{wide_bar} {pos}/{len}\n{msg}"));
        Some(pb)
    } else {
        None
    };

    delete(target, user, namespace_id, keys, &progress_bar)?;

    if let Some(pb) = &progress_bar {
        pb.finish_with_message(&format!("deleted {} key value pairs", len));
    }

    StdOut::success("Success");
    Ok(())
}
