extern crate base64;

use std::fs;
use std::fs::metadata;
use std::path::Path;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;

use indicatif::{ProgressBar, ProgressStyle};

use crate::commands::kv::validate_target;
use crate::kv::bulk::put;
use crate::kv::bulk::MAX_PAIRS;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message;

use super::bulk_api_client;

pub fn run(
    target: &Target,
    user: &GlobalUser,
    namespace_id: &str,
    filename: &Path,
) -> Result<(), failure::Error> {
    validate_target(target)?;

    let mut pairs: Vec<KeyValuePair> = match &metadata(filename) {
        Ok(file_type) if file_type.is_file() => {
            let data = fs::read_to_string(filename)?;
            let data_vec = serde_json::from_str(&data);
            match data_vec {
                Ok(data_vec) => Ok(data_vec),
                Err(_) => Err(failure::format_err!("Failed to decode JSON. Please make sure to follow the format, [{{\"key\": \"test_key\", \"value\": \"test_value\"}}, ...]"))
            }
        }
        Ok(_) => Err(failure::format_err!(
            "{} should be a JSON file, but is not",
            filename.display()
        )),
        Err(e) => Err(failure::format_err!("{}", e)),
    }?;

    let len = pairs.len();

    message::working(&format!("uploading {} key value pairs", len));
    let progress_bar = if len > MAX_PAIRS {
        let pb = ProgressBar::new(len as u64);
        pb.set_style(ProgressStyle::default_bar().template("{wide_bar} {pos}/{len}\n{msg}"));
        Some(pb)
    } else {
        None
    };

    let client = bulk_api_client(user)?;
    while !pairs.is_empty() {
        let p: Vec<KeyValuePair> = if pairs.len() > MAX_PAIRS {
            pairs.drain(0..MAX_PAIRS).collect()
        } else {
            pairs.drain(0..).collect()
        };

        put(&client, target, namespace_id, &p)?;

        if let Some(pb) = &progress_bar {
            pb.inc(p.len() as u64);
        }
    }

    if let Some(pb) = &progress_bar {
        pb.finish_with_message(&format!("uploaded {} key value pairs", len));
    }

    message::success("Success");
    Ok(())
}
