use std::num::ParseIntError;

use anyhow::{anyhow, Result};
use cloudflare::framework::auth::Credentials;
use http::HeaderMap;
use http::HeaderValue;
use http::Method;
use serde_json::value::Value as JsonValue;

use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeys;
use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeysParams;
use cloudflare::endpoints::workerskv::Key;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClient;

use crate::settings::global_user::GlobalUser;
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

// since the value returned is just a naked string, and the TTL is in a response header,
// we can't use the API crate for this :(
pub fn get_value(
    key: &str,
    namespace: &str,
    account_id: &str,
    user: &GlobalUser,
    client: &reqwest::blocking::Client,
) -> Result<(String, Option<i64>)> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        account_id, namespace, key
    );

    let mut headers = HeaderMap::new();

    for (header_key, header_str) in Credentials::from(user.clone()).headers() {
        let header_value = HeaderValue::from_str(&header_str)?;
        headers.append(header_key, header_value);
    }

    let request = client.request(Method::GET, url).headers(headers).build()?;
    let response = client.execute(request)?;

    let expiration = response
        .headers()
        .get("Expiration")
        .map(|header| match header.to_str() {
            Ok(s) => s.parse().map_err(|e: ParseIntError| anyhow!(e)),
            Err(e) => Err(anyhow!(e)),
        })
        .transpose()?;

    let value = response.text()?;

    Ok((value, expiration))
}
