use anyhow::Result;

use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Manifest;
use crate::terminal::message::{Message, StdOut};

use cloudflare::endpoints::r2::{CreateBucket, DeleteBucket, ListBuckets};
use cloudflare::framework::apiclient::ApiClient;

pub fn list(manifest: &Manifest, env: Option<&str>, user: &GlobalUser) -> Result<()> {
    let account_id = manifest.get_account_id(env)?;
    let client = http::cf_v4_client(user)?;
    let result = client.request(&ListBuckets {
        account_identifier: &account_id,
    });

    match result {
        Ok(response) => {
            let buckets: Vec<String> = response
                .result
                .buckets
                .into_iter()
                .map(|b| b.name)
                .collect();
            println!("{:?}", buckets);
        }
        Err(e) => println!("{}", e),
    }

    Ok(())
}

pub fn create(manifest: &Manifest, env: Option<&str>, user: &GlobalUser, name: &str) -> Result<()> {
    let account_id = manifest.get_account_id(env)?;
    let msg = format!("Creating bucket \"{}\"", name);
    StdOut::working(&msg);

    let client = http::cf_v4_client(user)?;
    let result = client.request(&CreateBucket {
        account_identifier: &account_id,
        bucket_name: name,
    });

    match result {
        Ok(_) => {
            StdOut::success("Success!");
        }
        Err(e) => print!("{}", e),
    }

    Ok(())
}

pub fn delete(manifest: &Manifest, env: Option<&str>, user: &GlobalUser, name: &str) -> Result<()> {
    let account_id = manifest.get_account_id(env)?;
    let msg = format!("Deleting bucket \"{}\"", name);
    StdOut::working(&msg);

    let client = http::cf_v4_client(user)?;
    let result = client.request(&DeleteBucket {
        account_identifier: &account_id,
        bucket_name: name,
    });

    match result {
        Ok(_) => {
            StdOut::success("Success!");
        }
        Err(e) => print!("{}", e),
    }

    Ok(())
}
