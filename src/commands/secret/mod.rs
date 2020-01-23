// use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::emoji;
use crate::terminal::message;
// use cloudflare::endpoints::workers::create_secret::CreateSecret;
use crate::commands::kv;
use crate::http;
use cloudflare::endpoints::workers::create_secret::CreateSecret;
use cloudflare::endpoints::workers::create_secret::CreateSecretParams;
use cloudflare::endpoints::workers::delete_secret::DeleteSecret;
use cloudflare::endpoints::workers::WorkersSecret;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};
use cloudflare::framework::{HttpApiClient, HttpApiClientConfig};

// For interactively handling  reading in a string
pub fn interactive_get_string(prompt_string: &str) -> String {
    println!("{}", prompt_string);
    let foo: String = read!("{}\n");
    // println!("{}", answer);
    // read!("{}\n").as_str()
    foo
}

fn format_error(e: ApiFailure) -> String {
    print!("Will remove, API Failure details {}", e);
    // e.code
    http::format_error(e, Some(&secret_errors))
}
// secret_errors() provides more detailed explanations of Workers KV API error codes.
// See https://api.cloudflare.com/#workers-secrets ? for details.
fn secret_errors(error_code: u16) -> &'static str {
    // TODO replace these with real error messages
    match error_code {
        7003 | 7000 => {
            "Your wrangler.toml is likely missing the field \"account_id\", which is required to write to Workers KV."
        }
        // namespace errors
        10010 | 10011 | 10012 | 10013 | 10014 | 10018 => {
            "Run `wrangler kv:namespace list` to see your existing namespaces with IDs"
        }
        10009 => "Run `wrangler kv:key list` to see your existing keys", // key errors
        // TODO: link to more info
        // limit errors
        10022 | 10024 | 10030 => "See documentation",
        // TODO: link to tool for this?
        // legacy namespace errors
        10021 | 10035 | 10038 => "Consider moving this namespace",
        // cloudflare account errors
        10017 | 10026 => "Workers KV is a paid feature, please upgrade your account (https://www.cloudflare.com/products/workers-kv/)",
        10001 => "The content you passed in is not an excepted string. Try entering just a string",
        _ => "Unknown code",
    }
}
fn api_put_secret(
    user: &GlobalUser,
    target: &Target,
    name: &str,
    secret_value: String,
) -> Result<(), failure::Error> {
    let msg = format!("Creating the secret for script name {}", target.name);
    // let response = call_api(target, name, secret_value);
    let client = http::cf_v4_api_client(user, HttpApiClientConfig::default())?;

    let response = client.request(&CreateSecret {
        account_identifier: &target.account_id,
        script_name: &target.name,
        params: CreateSecretParams {
            name: name.to_string(),
            value: secret_value.to_string(),
        },
    });

    match response {
        // TODO: 201 if new secret, 200 if updated and report to user
        Ok(_) => message::success(&format!("Success! You've uploaded secret {}.", name)),
        Err(e) => failure::bail!(format!("Formatted error{}", format_error(e))),
        (_) => print!("some unknown format"),
    }
    message::working(&msg);
    Ok(())
    // message::success(&format!("Success! You've uploaded secret {}.", name));
}
fn api_delete_secret(user: &GlobalUser, target: &Target, name: &str) -> Result<(), failure::Error> {
    let msg = format!("Deleting the secret {} on script {}.", name, target.name);
    // let response = call_api(target, name, secret_value);
    let client = http::cf_v4_api_client(user, HttpApiClientConfig::default())?;

    let response = client.request(&DeleteSecret {
        account_identifier: &target.account_id,
        script_name: &target.name,
        secret_name: name,
    });

    match response {
        // TODO: 201 if new secret, 200 if updated and report to user
        Ok(_) => message::success(&format!("You've deleted the secret {}.", name)),
        Err(e) => failure::bail!(format!("Formatted error{}", format_error(e))),
        (_) => print!("some unknown format"),
    }
    message::working(&msg);
    Ok(())
    // message::success(&format!("Success! You've uploaded secret {}.", name));
}

pub fn create_secret(name: &str, user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    let secret_value = interactive_get_string(&format!(
        "Enter the secret text you'd like assigned to the variable {} on the script named {}",
        name, target.name
    ));
    if secret_value.is_empty() {
        failure::bail!(format!("Enter a non empty string."))
    }
    if target.account_id.is_empty() {
        failure::bail!(format!(
            "{} You must provide an account_id in your wrangler.toml before creating a secret!",
            emoji::WARN
        ))
    }
    api_put_secret(&user, &target, name, secret_value)
}
pub fn delete_secret(name: &str, user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    let secret_value = interactive_get_string(&format!(
        "Enter the secret text you'd like assigned to the variable {} on the script named {}",
        name, target.name
    ));
    if secret_value.is_empty() {
        failure::bail!(format!("Enter a non empty string."))
    }
    if target.account_id.is_empty() {
        failure::bail!(format!(
            "{} You must provide an account_id in your wrangler.toml before creating a secret!",
            emoji::WARN
        ))
    }
    api_put_secret(&user, &target, name, secret_value)
}
