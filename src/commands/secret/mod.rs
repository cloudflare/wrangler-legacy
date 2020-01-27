// use crate::http;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::emoji;
use crate::terminal::message;
use crate::terminal::utils;
// use cloudflare::endpoints::workerskv::list_namespaces::ListNamespaces;
// use cloudflare::endpoints::workerskv::list_namespaces::ListNamespacesParams;
use crate::http;
use cloudflare::endpoints::workers::{CreateSecret, CreateSecretParams, DeleteSecret, ListSecrets};
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::HttpApiClientConfig;

fn format_error(e: ApiFailure) -> String {
    http::format_error(e, Some(&secret_errors))
}

fn validate_target(target: &Target) -> Result<(), failure::Error> {
    let mut missing_fields = Vec::new();

    if target.account_id.is_empty() {
        missing_fields.push("account_id")
    };

    if !missing_fields.is_empty() {
        failure::bail!(
            "{} Your wrangler.toml is missing the following field(s): {:?}",
            emoji::WARN,
            missing_fields
        )
    } else {
        Ok(())
    }
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
        10007 => "The name does not exist on that script. Are you sure you entered the correct environment? account ID?", // Verify
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
    let client = http::cf_v4_api_client(user, HttpApiClientConfig::default())?;

    let response = client.request(&CreateSecret {
        account_identifier: &target.account_id,
        script_name: &target.name,
        params: CreateSecretParams {
            name: name.to_string(),
            text: secret_value.to_string(),
            secret_type: "secret_text".to_string(),
        },
    });
    match response {
        // TODO: 201 if new secret, 200 if updated and report to user
        Ok(_) => message::success(&format!("Success! You've uploaded secret {}.", name)),
        Err(e) => failure::bail!(format!("Formatted error{}", format_error(e))),
    }
    message::working(&msg);
    Ok(())
    // message::success(&format!("Success! You've uploaded secret {}.", name));
}
fn api_delete_secret(user: &GlobalUser, target: &Target, name: &str) -> Result<(), failure::Error> {
    let msg = format!("Deleting the secret {} on script {}.", name, target.name);
    let client = http::cf_v4_api_client(user, HttpApiClientConfig::default())?;

    let response = client.request(&DeleteSecret {
        account_identifier: &target.account_id,
        script_name: &target.name,
        secret_name: name,
    });

    match response {
        Ok(_) => message::success(&format!("You've deleted the secret {}.", name)),
        Err(e) => failure::bail!(format!(
            "TODO: delete this print of the error {}",
            format_error(e)
        )),
    }
    message::working(&msg);
    Ok(())
}
fn api_get_secrets(user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    let client = http::cf_v4_api_client(user, HttpApiClientConfig::default())?;

    let response = client.request(&ListSecrets {
        account_identifier: &target.account_id,
        script_name: &target.name,
    });
    // TODO remove this is for
    // Testing with KV namespaces endpoint
    // let params = ListNamespacesParams {
    //     page: Some(1),
    //     per_page: Some(12),
    // };

    // let response = client.request(&ListNamespaces {
    //     account_identifier: &target.account_id,
    //     params,
    // });

    match response {
        Ok(_success) => {
            let secrets = _success.result;
            println!("{}", serde_json::to_string(&secrets)?);
        }
        Err(e) => failure::bail!("{}", format_error(e)),
    }
    Ok(())
}

pub fn create_secret(name: &str, user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    validate_target(target)?;

    let secret_value = utils::interactive_get_string(&format!(
        "Enter the secret text you'd like assigned to the variable {} on the script named {}",
        name, target.name
    ));
    if secret_value.is_empty() {
        failure::bail!(format!("Enter a non empty string."))
    }
    api_put_secret(&user, &target, name, secret_value)
}
pub fn delete_secret(name: &str, user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    validate_target(target)?;

    match utils::interactive_delete(&format!(
        "Are you sure you want to permentally delete the variable {} on the script named {}",
        name, target.name
    )) {
        Ok(true) => (),
        Ok(false) => {
            message::info(&format!("Not deleting secret {}", name));
            return Ok(());
        }
        Err(e) => failure::bail!(e),
    }

    api_delete_secret(&user, &target, name)
}
pub fn list_secrets(user: &GlobalUser, target: &Target) -> Result<(), failure::Error> {
    validate_target(target)?;
    api_get_secrets(&user, &target)
}
