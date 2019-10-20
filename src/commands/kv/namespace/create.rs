use cloudflare::endpoints::workerskv::create_namespace::CreateNamespace;
use cloudflare::endpoints::workerskv::create_namespace::CreateNamespaceParams;
use cloudflare::endpoints::workerskv::WorkersKvNamespace;
use cloudflare::framework::apiclient::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;
use regex::Regex;

pub fn create(
    target: &Target,
    env: Option<&str>,
    user: &GlobalUser,
    binding: &str,
) -> Result<(), failure::Error> {
    kv::validate_target(target)?;
    validate_binding(binding)?;

    let title = format!("{}-{}", target.name, binding);
    let msg = format!("Creating namespace with title \"{}\"", title);
    message::working(&msg);

    let client = kv::api_client(user)?;
    let result = call_api(&client, target, &title);

    match result {
        Ok(success) => {
            let namespace = success.result;
            message::success(&format!("Success: {:#?}", namespace));
            match target.kv_namespaces {
                None => {
                    match env {
                        Some(env) => message::success(&format!(
                            "Add the following to your wrangler.toml under [env.{}]:",
                            env
                        )),
                        None => message::success("Add the following to your wrangler.toml:"),
                    };
                    println!(
                        "kv-namespaces = [ \n\
                         \t {{ binding = \"{}\", id = \"{}\" }} \n\
                         ]",
                        binding, namespace.id
                    );
                }
                Some(_) => {
                    match env {
                        Some(env) => message::success(&format!(
                            "Add the following to your wrangler.toml's \"kv-namespaces\" array in [env.{}]:",
                            env
                        )),
                        None => message::success("Add the following to your wrangler.toml's \"kv-namespaces\" array:"),
                    };
                    println!("{{ binding = \"{}\", id = \"{}\" }}", binding, namespace.id);
                }
            }
        }
        Err(e) => print!("{}", kv::format_error(e)),
    }

    Ok(())
}

pub fn call_api(
    client: &impl ApiClient,
    target: &Target,
    title: &str,
) -> Result<ApiSuccess<WorkersKvNamespace>, ApiFailure> {
    client.request(&CreateNamespace {
        account_identifier: &target.account_id,
        params: CreateNamespaceParams {
            title: title.to_string(),
        },
    })
}

fn validate_binding(binding: &str) -> Result<(), failure::Error> {
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    if !re.is_match(binding) {
        failure::bail!(
            "A binding can only have alphanumeric and _ characters, and cannot begin with a number"
        )
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_detect_invalid_binding() {
        let invalid_bindings = vec!["hi there", "1234"];
        for binding in invalid_bindings {
            assert!(validate_binding(binding).is_err());
        }
    }

    #[test]
    fn it_can_detect_valid_binding() {
        let valid_bindings = vec!["ONE", "TWO_TWO", "__private_variable", "rud3_var"];
        for binding in valid_bindings {
            assert!(validate_binding(binding).is_ok());
        }
    }
}
