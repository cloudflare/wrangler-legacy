use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeys;
use cloudflare::endpoints::workerskv::list_namespace_keys::ListNamespaceKeysParams;
use cloudflare::endpoints::workerskv::Key;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClient;

use serde_json::value::Value as JsonValue;

use crate::settings::project::Project;

pub struct KeyList {
    keys_result: Option<Vec<Key>>,
    prefix: Option<String>,
    client: HttpApiClient,
    project: Project,
    namespace_id: String,
    cursor: Option<String>,
    pub error: Option<ApiFailure>,
}

impl KeyList {
    pub fn fetch(
        project: &Project,
        client: HttpApiClient,
        namespace_id: &str,
        prefix: Option<&str>,
    ) -> KeyList {
        KeyList {
            keys_result: None,
            prefix: prefix.map(str::to_string),
            client,
            project: project.to_owned(),
            namespace_id: namespace_id.to_string(),
            cursor: None,
            error: None,
        }
    }

    fn request_params(&self) -> ListNamespaceKeys {
        ListNamespaceKeys {
            account_identifier: &self.project.account_id,
            namespace_identifier: &self.namespace_id,
            params: self.params(),
        }
    }

    fn params(&self) -> ListNamespaceKeysParams {
        ListNamespaceKeysParams {
            limit: None, // Defaults to 1000 (the maximum)
            cursor: None,
            prefix: self.prefix.to_owned(),
        }
    }

    fn get_batch(&mut self) -> Option<Result<Key, ApiFailure>> {
        let response = self.client.request(&self.request_params());

        let mut result;
        let mut error = None;

        match response {
            Ok(success) => {
                result = success.result;
                self.cursor = extract_cursor(success.result_info.clone());
            }
            Err(e) => {
                result = Vec::new();
                error = Some(e);
            }
        };

        if let Some(error) = error {
            Some(Err(error))
        } else {
            let key = result.pop()?;
            self.keys_result = Some(result);

            Some(Ok(key))
        }
    }
}

impl Iterator for KeyList {
    type Item = Result<Key, ApiFailure>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.keys_result.to_owned() {
            Some(mut keys) => {
                let key = keys.pop();
                self.keys_result = Some(keys);

                if key.is_none() {
                    if self.cursor.is_none() {
                        None
                    } else {
                        self.get_batch()
                    }
                } else {
                    Some(Ok(key?))
                }
            }
            // if this is None, we have not made a request yet
            None => self.get_batch(),
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
