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
                        None => {
                            message::success(&format!("Add the following to your wrangler.toml:"))
                        }
                    };
                    println!(
                        "kv-namespaces = [ \n\
                         \t {{ binding: \"{}\", id: \"{}\" }} \n\
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
                        None => message::success(&format!("Add the following to your wrangler.toml's \"kv-namespaces\" array:")),
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
