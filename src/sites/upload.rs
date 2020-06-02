use indicatif::ProgressBar;

use cloudflare::endpoints::workerskv::write_bulk::KeyValuePair;
use cloudflare::framework::apiclient::ApiClient;

use crate::http;
use crate::kv::bulk::put;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message;

// The consts below are halved from the API's true capacity to help avoid
// hammering it with large requests.
const PAIRS_MAX_COUNT: usize = 5000;
const UPLOAD_MAX_SIZE: usize = 50 * 1024 * 1024;

pub fn upload_files(
    target: &Target,
    user: &GlobalUser,
    namespace_id: &str,
    mut pairs: Vec<KeyValuePair>,
) -> Result<(), failure::Error> {
    if !pairs.is_empty() {
        let client = http::cf_v4_client(user)?;
        // Iterate over all key-value pairs and create batches of uploads, each of which are
        // maximum 5K key-value pairs in size OR maximum ~50MB in size. Upload each batch
        // as it is created.
        let mut key_count = 0;
        let mut key_pair_bytes = 0;
        let mut key_value_batch: Vec<KeyValuePair> = Vec::new();

        message::working("Uploading site files");
        let pb = if pairs.len() > PAIRS_MAX_COUNT {
            Some(ProgressBar::new(pairs.len() as u64))
        } else {
            None
        };
        while !(pairs.is_empty() && key_value_batch.is_empty()) {
            if pairs.is_empty() {
                // Last batch to upload
                upload_batch(&client, target, namespace_id, &mut key_value_batch)?;
            } else {
                let pair = pairs.pop().unwrap();
                if key_count + 1 > PAIRS_MAX_COUNT
                // Keep upload size small to keep KV bulk API happy
                || key_pair_bytes + pair.key.len() + pair.value.len() > UPLOAD_MAX_SIZE
                {
                    upload_batch(&client, target, namespace_id, &mut key_value_batch)?;
                    if let Some(p) = &pb {
                        p.inc(key_value_batch.len() as u64);
                    }

                    // If upload successful, reset counters
                    key_count = 0;
                    key_pair_bytes = 0;
                }

                // Add the popped key-value pair to the running batch of key-value pair uploads
                key_count += 1;
                key_pair_bytes = key_pair_bytes + pair.key.len() + pair.value.len();
                key_value_batch.push(pair);
            }
        }
        if let Some(p) = pb {
            p.finish_with_message("Done Uploading");
        }
    }

    Ok(())
}

fn upload_batch(
    client: &impl ApiClient,
    target: &Target,
    namespace_id: &str,
    key_value_batch: &mut Vec<KeyValuePair>,
) -> Result<(), failure::Error> {
    // If partial upload fails (e.g. server error), return that error message
    put(client, target, namespace_id, &key_value_batch)?;
    // Can clear batch now that we've uploaded it
    key_value_batch.clear();
    Ok(())
}
