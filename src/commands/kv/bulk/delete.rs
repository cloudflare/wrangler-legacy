extern crate base64;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use indicatif::{ProgressBar, ProgressStyle};

use crate::commands::kv;
use crate::kv::bulk::delete;
use crate::kv::bulk::MAX_PAIRS;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::interactive;
use crate::terminal::message;

pub fn run(
    target: &Target,
    user: &GlobalUser,
    namespace_id: &str,
    filename: &Path,
) -> Result<(), failure::Error> {
    kv::validate_target(target)?;

    match interactive::confirm(&format!(
        "Are you sure you want to delete all keys in {}?",
        filename.display()
    )) {
        Ok(true) => (),
        Ok(false) => {
            message::info(&format!("Not deleting keys in {}", filename.display()));
            return Ok(());
        }
        Err(e) => failure::bail!(e),
    }

    let pairs: Result<Vec<KeyValuePair>, failure::Error> = match &metadata(filename) {
        Ok(file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            let keys_vec = serde_json::from_str(&data);
            match keys_vec {
                Ok(keys_vec) => Ok(keys_vec),
                Err(_) => failure::bail!("Failed to decode JSON. Please make sure to follow the format, [{\"key\": \"test_key\", \"value\": \"test_value\"}, ...]")
            }
        }
        Ok(_) => failure::bail!("{} should be a JSON file, but is not", filename.display()),
        Err(e) => failure::bail!("{}", e),
    };

    let mut keys: Vec<String> = pairs?.iter().map(|kv| kv.key.to_owned()).collect();

    let len = keys.len();
    message::working(&format!("deleting {} key value pairs", len));
    let progress_bar = if len > MAX_PAIRS {
        let pb = ProgressBar::new(len as u64);
        pb.set_style(ProgressStyle::default_bar().template("{wide_bar} {pos}/{len}\n{msg}"));
        Some(pb)
    } else {
        None
    };

    while !keys.is_empty() {
        let p: Vec<String> = if keys.len() > MAX_PAIRS {
            keys.drain(0..MAX_PAIRS).collect()
        } else {
            keys.drain(0..).collect()
        };

        if let Err(e) = delete(target, user, namespace_id, p) {
            failure::bail!("{}", e);
        }

        if let Some(pb) = &progress_bar {
            pb.inc(keys.len() as u64);
        }
    }

    if let Some(pb) = &progress_bar {
        pb.finish_with_message(&format!("deleted {} key value pairs", len));
    }

    message::success("Success");
    Ok(())
}
