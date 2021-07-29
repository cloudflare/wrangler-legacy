// TODO: (gabbi) This file should use cloudflare-rs instead of our http::legacy_auth_client
// when https://github.com/cloudflare/cloudflare-rs/issues/26 is handled (this is
// because the SET key request body is not json--it is the raw value).

use std::fs;
use std::fs::metadata;

use anyhow::Result;
use cloudflare::framework::response::ApiFailure;
use url::Url;

use crate::commands::kv;
use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdOut};
use regex::Regex;
use reqwest::blocking::multipart;

pub struct KVMetaData {
    pub namespace_id: String,
    pub key: String,
    pub value: String,
    pub is_file: bool,
    pub expiration: Option<String>,
    pub expiration_ttl: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub fn parse_metadata(arg: Option<&str>) -> Result<Option<serde_json::Value>> {
    match arg {
        None => Ok(None),
        Some(s) => {
            match serde_json::from_str(s) {
                Ok(v) => Ok(Some(v)),
                Err(e) => {
                    // try to help users that forget to double-quote a JSON string
                    let re = Regex::new(r#"^['"]?[^"'{}\[\]]*['"]?$"#)?;
                    if re.is_match(s) {
                        anyhow::bail!(
                            "did you remember to double quote strings, like --metadata '\"made with 🤠 wrangler\"'"
                        )
                    }
                    anyhow::bail!(e.to_string())
                }
            }
        }
    }
}

pub fn put(target: &Target, user: &GlobalUser, data: KVMetaData) -> Result<()> {
    let api_endpoint = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        target.account_id.load()?,
        &data.namespace_id,
        kv::url_encode_key(&data.key)
    );

    // Add expiration and expiration_ttl query options as necessary.
    let mut query_params: Vec<(&str, &str)> = vec![];

    if let Some(exp) = &data.expiration {
        query_params.push(("expiration", exp))
    };
    if let Some(ttl) = &data.expiration_ttl {
        query_params.push(("expiration_ttl", ttl))
    };
    let url = Url::parse_with_params(&api_endpoint, query_params)?;

    let res = get_response(data, user, &url)?;

    let response_status = res.status();
    if response_status.is_success() {
        StdOut::success("Success")
    } else {
        // This is logic pulled from cloudflare-rs for pretty error formatting right now;
        // it will be redundant when we switch to using cloudflare-rs for all API requests.
        let parsed = res.json();
        let errors = parsed.unwrap_or_default();
        print!(
            "{}",
            kv::format_error(ApiFailure::Error(response_status, errors))
        );
    }

    Ok(())
}

fn get_response(
    data: KVMetaData,
    user: &GlobalUser,
    url: &Url,
) -> Result<reqwest::blocking::Response> {
    let url_into_str = url.to_string();
    let client = http::legacy_auth_client(user);
    let value_body = get_request_body(&data)?;
    let res = match &data.metadata {
        Some(metadata) => {
            let value_part = multipart::Part::bytes(value_body);
            let form = multipart::Form::new()
                .part("value", value_part)
                .text("metadata", metadata.to_string());
            client.put(&url_into_str).multipart(form).send()?
        }
        None => client.put(&url_into_str).body(value_body).send()?,
    };
    Ok(res)
}

// If is_file is true, overwrite value to be the contents of the given
// filename in the 'value' arg.
fn get_request_body(data: &KVMetaData) -> Result<Vec<u8>> {
    if data.is_file {
        match &metadata(&data.value) {
            Ok(file_type) if file_type.is_file() => Ok(fs::read(&data.value)?),
            Ok(file_type) if file_type.is_dir() => anyhow::bail!(
                "--path argument takes a file, {} is a directory",
                data.value
            ),
            Ok(_) => anyhow::bail!(
                "--path argument points to an entity that is not a file or a directory: {}",
                data.value
            ),
            Err(e) => anyhow::bail!("{}", e),
        }
    } else {
        Ok(data.value.clone().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_parser_legal() {
        for input in &[
            "true",
            "false",
            "123.456",
            r#""some string""#,
            "[1, 2]",
            "{\"key\": \"value\"}",
        ] {
            assert!(parse_metadata(Some(input)).is_ok());
        }
    }

    #[test]
    fn metadata_parser_illegal() {
        for input in &["something", "{key: 123}", "[1, 2"] {
            assert!(parse_metadata(Some(input)).is_err());
        }
    }

    #[test]
    fn metadata_parser_error_message_unquoted_string_error_message() -> Result<(), &'static str> {
        for input in &["abc", "'abc'", "'abc", "abc'", "\"abc", "abc\""] {
            match parse_metadata(Some(input)) {
                Ok(_) => return Err("illegal value was parsed successfully"),
                Err(e) => {
                    let expected_message = "did you remember to double quote strings, like --metadata '\"made with 🤠 wrangler\"'";
                    assert_eq!(expected_message, e.to_string());
                }
            }
        }
        Ok(())
    }
}
