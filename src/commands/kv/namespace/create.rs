use regex::Regex;

use crate::commands::kv;
use crate::http;
use crate::kv::namespace::create;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::terminal::message;

pub fn run(
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

    let client = http::cf_v4_client(user)?;
    let result = create(&client, target, &title);

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
