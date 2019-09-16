use cloudflare::endpoints::workerskv::create_namespace::CreateNamespace;
use cloudflare::endpoints::workerskv::create_namespace::CreateNamespaceParams;
use cloudflare::framework::apiclient::ApiClient;

use crate::commands::kv;
use crate::settings::global_user::GlobalUser;
use crate::settings::target::Target;
use crate::terminal::message;

pub fn create(
    target: &Target,
    env: Option<&str>,
    user: GlobalUser,
    binding: &str,
) -> Result<(), failure::Error> {
    let client = kv::api_client(user)?;

    if !validate_binding(binding) {
        failure::bail!(
            "A binding can only have alphanumeric and _ characters, and cannot begin with a number"
        );
    }

    let title = format!("{}-{}", target.name, binding);
    let msg = format!("Creating namespace with title \"{}\"", title);
    message::working(&msg);

    let response = client.request(&CreateNamespace {
        account_identifier: &target.account_id,
        params: CreateNamespaceParams {
            title: title.to_string(),
        },
    });

    match response {
        Ok(success) => {
            message::success(&format!("Success: {:#?}", success.result));
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
                        binding, success.result.id
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
                    println!(
                        "{{ binding: \"{}\", id: \"{}\" }}",
                        binding, success.result.id
                    );
                }
            }
        }
        Err(e) => kv::print_error(e),
    }

    Ok(())
}

fn validate_binding(binding: &str) -> bool {
    use regex::Regex;
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    re.is_match(binding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_detect_invalid_binding() {
        let invalid_bindings = vec!["hi there", "1234"];
        for binding in invalid_bindings {
            assert!(!validate_binding(binding));
        }
    }

    #[test]
    fn it_can_detect_valid_binding() {
        let valid_bindings = vec!["ONE", "TWO_TWO", "__private_variable", "rud3_var"];
        for binding in valid_bindings {
            assert!(validate_binding(binding));
        }
    }
}
