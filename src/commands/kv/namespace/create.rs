use anyhow::Result;
use regex::Regex;

use crate::commands::kv;
use crate::http;
use crate::kv::namespace::create;
use crate::settings::global_user::GlobalUser;
use crate::settings::toml::{ConfigKvNamespace, KvNamespace, Manifest};
use crate::terminal::message::{Message, StdOut};
pub fn run(
    manifest: &Manifest,
    is_preview: bool,
    env: Option<&str>,
    user: &GlobalUser,
    binding: &str,
) -> Result<()> {
    let account_id = manifest.get_account_id(env)?;
    let worker_name = manifest.worker_name(env);
    validate_binding(binding)?;

    let mut title = format!("{}-{}", worker_name, binding);
    if is_preview {
        title.push_str("_preview");
    }
    let msg = format!("Creating namespace with title \"{}\"", title);
    StdOut::working(&msg);

    let client = http::cf_v4_client(user)?;
    let result = create(&client, &account_id, &title);

    match result {
        Ok(success) => {
            let namespace = success.result;
            StdOut::success("Success!");
            println!(
                "{}",
                toml_modification_instructions(
                    KvNamespace {
                        binding: binding.to_string(),
                        id: namespace.id,
                    },
                    manifest.kv_namespaces.as_ref(),
                    env,
                    is_preview,
                )
            );
        }
        Err(e) => print!("{}", kv::format_error(e)),
    }

    Ok(())
}

fn validate_binding(binding: &str) -> Result<()> {
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    if !re.is_match(binding) {
        anyhow::bail!(
            "A binding can only have alphanumeric and _ characters, and cannot begin with a number"
        )
    }
    Ok(())
}

fn toml_modification_instructions(
    new_namespace: KvNamespace,
    all_namespaces: Option<&Vec<ConfigKvNamespace>>,
    env: Option<&str>,
    is_preview: bool,
) -> String {
    let mut msg = "Add the following to your configuration file".to_string();

    if all_namespaces.is_some() {
        msg.push_str(" in your kv_namespaces array");
    }

    if let Some(env) = env {
        msg.push_str(&format!(" under [env.{}]", env));
    }

    msg.push_str(":\n");

    let existing_namespace = if let Some(all_namespaces) = all_namespaces {
        all_namespaces
            .iter()
            .find(|namespace| namespace.binding == new_namespace.binding)
    } else {
        None
    };

    let mut inline_msg = format!("{{ binding = \"{}\", ", &new_namespace.binding);
    if let Some(existing_namespace) = existing_namespace {
        if is_preview {
            inline_msg.push_str(&format!("preview_id = \"{}\"", new_namespace.id));
            if let Some(existing_namespace_id) = &existing_namespace.id {
                inline_msg.push_str(&format!(", id = \"{}\"", existing_namespace_id));
            }
        } else {
            inline_msg.push_str(&format!("id = \"{}\"", new_namespace.id));
            if let Some(existing_namespace_preview_id) = &existing_namespace.preview_id {
                inline_msg.push_str(&format!(
                    ", preview_id = \"{}\"",
                    existing_namespace_preview_id
                ));
            }
        }
    } else {
        if is_preview {
            inline_msg.push_str("preview_id");
        } else {
            inline_msg.push_str("id")
        }
        inline_msg.push_str(" = \"");
        inline_msg.push_str(&new_namespace.id);
        inline_msg.push('\"');
    };
    inline_msg.push_str(" }");

    if all_namespaces.is_some() {
        msg.push_str(&inline_msg);
    } else {
        msg.push_str(&format!("kv_namespaces = [ \n\t {}\n]", &inline_msg));
    }

    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_messages_about_env() {
        let new_namespace = KvNamespace {
            id: "new_preview_id".to_string(),
            binding: "FOO".to_string(),
        };

        let all_namespaces = Some(vec![ConfigKvNamespace {
            binding: "BAR".to_string(),
            id: Some("production_id".to_string()),
            preview_id: None,
        }]);

        let env = Some("my_env");

        let is_preview = true;

        let msg =
            toml_modification_instructions(new_namespace, all_namespaces.as_ref(), env, is_preview);
        assert!(msg.contains("[env.my_env]"));
    }

    #[test]
    fn it_messages_about_preview() {
        let new_namespace = KvNamespace {
            id: "new_preview_id".to_string(),
            binding: "FOO".to_string(),
        };

        let all_namespaces = Some(vec![ConfigKvNamespace {
            binding: "FOO".to_string(),
            id: Some("existing_production_id".to_string()),
            preview_id: None,
        }]);

        let env = None;

        let is_preview = true;

        let msg =
            toml_modification_instructions(new_namespace, all_namespaces.as_ref(), env, is_preview);
        assert!(msg.contains("{ binding = \"FOO\", preview_id = \"new_preview_id\", id = \"existing_production_id\" }"));
        assert!(!msg.contains("kv_namespaces = ["));
    }

    #[test]
    fn it_messages_about_namespaces() {
        let new_namespace = KvNamespace {
            id: "new_id".to_string(),
            binding: "FOO".to_string(),
        };

        let all_namespaces = None;

        let env = None;

        let is_preview = false;

        let msg =
            toml_modification_instructions(new_namespace, all_namespaces.as_ref(), env, is_preview);
        assert!(msg.contains("{ binding = \"FOO\", id = \"new_id\" }"));
        assert!(msg.contains("kv_namespaces = ["));
    }

    #[test]
    fn it_doesnt_message_about_namespaces() {
        let new_namespace = KvNamespace {
            id: "new_id".to_string(),
            binding: "FOO".to_string(),
        };

        let all_namespaces = Some(vec![]);

        let env = None;

        let is_preview = false;

        let msg =
            toml_modification_instructions(new_namespace, all_namespaces.as_ref(), env, is_preview);
        assert!(msg.contains("{ binding = \"FOO\", id = \"new_id\" }"));
        assert!(!msg.contains("kv_namespaces = ["));
    }

    #[test]
    fn it_messages_about_overridden_namespaces() {
        let new_namespace = KvNamespace {
            id: "new_preview_id".to_string(),
            binding: "FOO".to_string(),
        };

        let all_namespaces = Some(vec![
            ConfigKvNamespace {
                binding: "FOO".to_string(),
                id: Some("existing_production_id".to_string()),
                preview_id: Some("existing_preview_id".to_string()),
            },
            ConfigKvNamespace {
                binding: "BAR".to_string(),
                id: Some("some_prod_id".to_string()),
                preview_id: None,
            },
        ]);

        let env = None;

        let is_preview = true;

        let msg =
            toml_modification_instructions(new_namespace, all_namespaces.as_ref(), env, is_preview);
        assert!(msg.contains("{ binding = \"FOO\", preview_id = \"new_preview_id\", id = \"existing_production_id\" }"));
        assert!(!msg.contains("kv_namespaces = ["));
    }

    #[test]
    fn it_messages_when_no_existing_id() {
        let new_namespace = KvNamespace {
            id: "new_preview_id".to_string(),
            binding: "FOO".to_string(),
        };

        let all_namespaces = Some(vec![
            ConfigKvNamespace {
                binding: "FOO".to_string(),
                id: None,
                preview_id: None,
            },
            ConfigKvNamespace {
                binding: "BAR".to_string(),
                id: Some("some_prod_id".to_string()),
                preview_id: None,
            },
        ]);

        let env = None;

        let is_preview = true;

        let msg =
            toml_modification_instructions(new_namespace, all_namespaces.as_ref(), env, is_preview);
        assert!(msg.contains("{ binding = \"FOO\", preview_id = \"new_preview_id\" }"));
        assert!(!msg.contains("kv_namespaces = ["));
    }

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
