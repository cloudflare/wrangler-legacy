use anyhow::Result;
use serde_json::value::Value as JsonValue;

use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeys;
use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeysParams;
use cloudflare::endpoints::workerskv::Key;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClient;

use crate::settings::toml::Target;

pub struct KeyList {
    keys_result: Option<Vec<Key>>,
    prefix: Option<String>,
    client: HttpApiClient,
    account_id: String,
    namespace_id: String,
    cursor: Option<String>,
    init_fetch: bool,
}

impl KeyList {
    pub fn new(
        target: &Target,
        client: HttpApiClient,
        namespace_id: &str,
        prefix: Option<&str>,
    ) -> Result<KeyList> {
        let iter = KeyList {
            keys_result: None,
            prefix: prefix.map(str::to_string),
            client,
            account_id: target.account_id.load()?.to_owned(),
            namespace_id: namespace_id.to_string(),
            cursor: None,
            init_fetch: false,
        };
        Ok(iter)
    }

    fn request_params(&self) -> ListNamespaceKeys {
        let params = ListNamespaceKeysParams {
            limit: None, // Defaults to 1000 (the maximum)
            cursor: self.cursor.to_owned(),
            prefix: self.prefix.to_owned(),
        };

        ListNamespaceKeys {
            account_identifier: &self.account_id,
            namespace_identifier: &self.namespace_id,
            params,
        }
    }

    fn get_batch(&mut self) -> Result<Vec<Key>, ApiFailure> {
        let response = self.client.request(&self.request_params());

        match response {
            Ok(success) => {
                self.cursor = extract_cursor(success.result_info.clone());
                log::info!("{:?}", self.cursor);
                Ok(success.result)
            }
            Err(e) => Err(e),
        }
    }
}

impl Iterator for KeyList {
    type Item = Result<Key, ApiFailure>;

    fn next(&mut self) -> Option<Self::Item> {
        // Attempt to extract next key from vector of keys in KeyList.
        // If no key vector or no keys left, go to fallback case below to
        // attempt to fetch the next page of keys from the Workers KV API.
        if let Some(mut keys) = self.keys_result.to_owned() {
            let key = keys.pop();
            self.keys_result = Some(keys);

            if let Some(k) = key {
                return Some(Ok(k));
            }
        }
        // Fallback case (if no remaining keys are found)
        if self.cursor.is_none() && self.init_fetch {
            None // Nothing left to fetch
        } else {
            if !self.init_fetch {
                // At this point, initial fetch is being performed.
                self.init_fetch = true;
            }
            match self.get_batch() {
                Ok(mut keys) => {
                    let key = keys.pop();
                    self.keys_result = Some(keys);
                    key.map(Ok)
                }
                Err(e) => Some(Err(e)),
            }
        }
    }
}

// Returns Some(cursor) if cursor is non-empty, otherwise returns None.
fn extract_cursor(result_info: Option<JsonValue>) -> Option<String> {
    let result_info = result_info.unwrap();
    let returned_cursor_value = &result_info["cursor"];
    let returned_cursor = returned_cursor_value.as_str().unwrap().to_string();
    if returned_cursor.is_empty() {
        None
    } else {
        Some(returned_cursor)
    }
}
