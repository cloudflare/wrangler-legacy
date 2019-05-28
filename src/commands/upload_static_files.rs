use percent_encoding::{percent_encode, PATH_SEGMENT_ENCODE_SET};
use walkdir::WalkDir;

use crate::settings::global_user::GlobalUser;
use crate::settings::project::Project;

use std::ffi::OsString;
use std::path::Path;

use serde::Deserialize;

use log::info;

pub fn upload_static_files(
    user: &GlobalUser,
    project: &Project,
    namespace: &str,
    directory: &str,
) -> Result<(), failure::Error> {
    println!("Uploading {} to {}", directory, namespace);

    // FIXME: we *could* use the bulk API here, and make less API calls. The trick is that we'd
    // have to calculate the total payload size, which is much harder, and we're not making that
    // many calls right now.
    for entry in WalkDir::new(directory) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            let key = generate_key(path, directory)?;

            let value = std::fs::read(path)?;

            info!("Uploading '{}'", path.display());

            upload_to_kv(user, &project.account_id, namespace, &key, value)?;
        }
    }

    Ok(())
}

pub fn generate_key(path: &Path, directory: &str) -> Result<String, failure::Error> {
    let path = path.strip_prefix(directory).unwrap();

    // next, we have to re-build the paths: if we're on Windows, we have paths with
    // `\` as separators. But we want to use `/` as separators. Because that's how URLs
    // work.
    let mut path_with_forward_slash = OsString::new();

    for (i, component) in path.components().enumerate() {
        // we don't want a leading `/`, so skip that
        if i > 0 {
            path_with_forward_slash.push("/");
        }

        path_with_forward_slash.push(component);
    }

    // if we have a non-utf8 path here, it will fail, but that's not realistically going to happen
    let path = path_with_forward_slash
        .to_str()
        .expect("found a non-UTF-8 path");
    let path_bytes = path.as_bytes();

    // we use PATH_SEGMENT_ENCODE_SET since we're encoding paths, this will turn / into %2F,
    // which is needed for the API call to put a `/` into the key.
    Ok(percent_encode(path_bytes, PATH_SEGMENT_ENCODE_SET).to_string())
}

fn upload_to_kv(user: &GlobalUser, account_id: &str, namespace_name: &str, key: &str, value: Vec<u8>) -> Result<(), failure::Error> {
    let namespace_id = fetch_namespace_id(user, account_id, namespace_name)?;

    let kv_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        account_id,
        namespace_id,
        key,
    );

    let client = reqwest::Client::new();

    info!("Attempting to upload '{}'", key);

    client
        .put(&kv_addr)
        .header("X-Auth-Key", &*user.api_key)
        .header("X-Auth-Email", &*user.email)
        .body(value)
        .send()?;

    Ok(())
}

fn fetch_namespace_id(user: &GlobalUser, account_id: &str, namespace_name: &str) -> Result<String, failure::Error> {
    let kv_addr = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces",
        account_id,
    );

    #[derive(Debug, Deserialize)]
    struct ApiResult {
        id: String,
        title: String,
    }

    #[derive(Debug, Deserialize)]
    struct ApiResponse {
        result: Vec<ApiResult>,
    }

    let client = reqwest::Client::new();

    info!("Attempting to find namespace '{}'", namespace_name);

    let mut request = client
        .get(&kv_addr)
        .header("X-Auth-Key", &*user.api_key)
        .header("X-Auth-Email", &*user.email)
        .send()?;

    let response: ApiResponse = request.json()?; 

    let id = response.result.iter().find(|&result| result.title == namespace_name).unwrap().id.clone();

    Ok(id)
}