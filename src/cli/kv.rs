use std::path::PathBuf;

use super::Cli;
use crate::commands;
use crate::commands::kv::key::{parse_metadata, KVMetaData};
use crate::settings::{global_user::GlobalUser, toml::Manifest};

use anyhow::{anyhow, Result};
use clap::ArgGroup;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum KvNamespace {
    /// Create a new namespace
    Create {
        /// The binding for your new namespace
        #[structopt(index = 1)]
        binding: String,
        /// Applies the command to the preview namespace
        #[structopt(name = "preview", long)]
        preview: bool,
    },
    /// Delete namespace
    Delete {
        #[structopt(flatten)]
        namespace: Namespace,
    },
    /// List all namespaces on your Cloudflare account
    List,
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(group = ArgGroup::with_name("namespace-specifier").required(true))]
pub struct Namespace {
    /// The binding of the namespace this action applies to
    #[structopt(long, short = "b", group = "namespace-specifier")]
    pub binding: Option<String>,

    /// Applies the command to the preview namespace when combined with --binding
    #[structopt(long, requires = "binding")]
    pub preview: bool,

    /// The ID of the namespace this action applies to
    #[structopt(
        name = "namespace-id",
        long,
        short = "n",
        group = "namespace-specifier"
    )]
    pub namespace_id: Option<String>,
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum KvKey {
    /// Put a key-value pair into a namespace
    Put {
        #[structopt(flatten)]
        namespace: Namespace,

        /// Key to write the value to
        #[structopt(name = "key", index = 1)]
        key: String,

        /// Value for key
        #[structopt(name = "value", index = 2)]
        value: String,

        /// Number of seconds for which the entries should be visible before they expire.
        /// At least 60. Takes precedence over 'expiration' option.
        #[structopt(name = "expiration-ttl", short = "t", long = "ttl")]
        expiration_ttl: Option<u64>,

        /// Number of seconds since the UNIX epoch, indicating when the key-value pair should expire.
        #[structopt(name = "expiration", long, short = "x")]
        expiration: Option<u64>,

        /// Arbitrary JSON to associate with a key-value pair. Must be no more than 1024 bytes.
        #[structopt(name = "metadata", long, short = "m")]
        metadata: Option<String>,

        /// The value passed in is a path to a file; open and upload its contents
        #[structopt(name = "path", long, short = "p")]
        path: bool,
    },
    /// Get a key's value from a namespace
    Get {
        #[structopt(flatten)]
        namespace: Namespace,

        /// Key whose value to get
        #[structopt(name = "key", index = 1)]
        key: String,
    },
    /// Delete a key and its value from a namespace
    Delete {
        #[structopt(flatten)]
        namespace: Namespace,

        /// Key whose value to get
        #[structopt(name = "key", index = 1)]
        key: String,
    },
    /// List all keys in a namespace. Produces JSON output
    List {
        #[structopt(flatten)]
        namespace: Namespace,

        /// The prefix for filtering listed keys
        #[structopt(name = "prefix", long, short = "p")]
        prefix: Option<String>,
    },
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "lower")]
pub enum KvBulk {
    /// Upload multiple key-value pairs to a namespace
    Put {
        #[structopt(flatten)]
        namespace: Namespace,

        /// The JSON file of key-value pairs to upload, in form [{\"key\":..., \"value\":...}\"...]
        #[structopt(index = 1)]
        path: PathBuf,
    },
    /// Delete multiple keys and their values from a namespace
    Delete {
        #[structopt(flatten)]
        namespace: Namespace,

        /// The JSON file of key-value pairs to upload, in form [\"<example-key>\", ...]
        #[structopt(index = 1)]
        path: PathBuf,
    },
}

pub fn kv_namespace(namespace: KvNamespace, cli_params: &Cli) -> Result<()> {
    let user = GlobalUser::new()?;
    let manifest = Manifest::new(&cli_params.config)?;
    let env = cli_params.environment.as_deref();

    match namespace {
        KvNamespace::Create { binding, preview } => {
            commands::kv::namespace::create(&manifest, preview, env, &user, &binding)
        }
        KvNamespace::Delete { namespace } => {
            let target = manifest.get_target(env, namespace.preview)?;
            let id = if let Some(binding) = namespace.binding {
                commands::kv::get_namespace_id(&target, &binding)?
            } else {
                namespace
                    .namespace_id
                    .expect("Namespace ID is required if binding isn't supplied")
            };
            commands::kv::namespace::delete(&target, &user, &id)
        }
        KvNamespace::List => {
            let target = manifest.get_target(env, false)?;
            commands::kv::namespace::list(&target, &user)
        }
    }
}

pub fn kv_key(key: KvKey, cli_params: &Cli) -> Result<()> {
    let user = GlobalUser::new()?;
    let manifest = Manifest::new(&cli_params.config)?;
    let env = cli_params.environment.as_deref();

    let target_and_namespace = |namespace: Namespace| -> Result<(_, _)> {
        let target = manifest.get_target(env, namespace.preview)?;
        let namespace_id = if let Some(binding) = namespace.binding {
            commands::kv::get_namespace_id(&target, &binding)?
        } else {
            namespace
                .namespace_id
                .expect("Namespace ID is required if binding isn't supplied")
        };
        Ok((target, namespace_id))
    };

    match key {
        KvKey::Get { namespace, key } => {
            let (target, namespace_id) = target_and_namespace(namespace)?;
            commands::kv::key::get(&target, &user, &namespace_id, &key)
        }
        KvKey::Put {
            namespace,
            key,
            value,
            path: is_file,
            expiration_ttl,
            expiration,
            metadata,
        } => {
            let (target, namespace_id) = target_and_namespace(namespace)?;
            let expiration = expiration.as_ref().map(ToString::to_string);
            let expiration_ttl = expiration_ttl.as_ref().map(ToString::to_string);
            let metadata = parse_metadata(metadata.as_deref())
                .map_err(|e| anyhow!("--metadata is not valid JSON: {}", e.to_string()))?;

            commands::kv::key::put(
                &target,
                &user,
                KVMetaData {
                    namespace_id,
                    key,
                    value,
                    is_file,
                    expiration,
                    expiration_ttl,
                    metadata,
                },
            )
        }
        KvKey::Delete { namespace, key } => {
            let (target, namespace_id) = target_and_namespace(namespace)?;
            commands::kv::key::delete(&target, &user, &namespace_id, &key)
        }
        KvKey::List { namespace, prefix } => {
            let (target, namespace_id) = target_and_namespace(namespace)?;
            commands::kv::key::list(&target, &user, &namespace_id, prefix.as_deref())
        }
    }
}

pub fn kv_bulk(bulk: KvBulk, cli_params: &Cli) -> Result<()> {
    // Get environment and bindings
    let manifest = Manifest::new(&cli_params.config)?;
    let user = GlobalUser::new()?;
    let env = cli_params.environment.as_deref();

    let target_and_namespace = |namespace: Namespace| -> Result<(_, _)> {
        let target = manifest.get_target(env, namespace.preview)?;
        let namespace_id = if let Some(binding) = namespace.binding {
            commands::kv::get_namespace_id(&target, &binding)?
        } else {
            namespace
                .namespace_id
                .expect("Namespace ID is required if binding isn't supplied")
        };
        Ok((target, namespace_id))
    };

    match bulk {
        KvBulk::Put { namespace, path } => {
            let (target, namespace_id) = target_and_namespace(namespace)?;
            commands::kv::bulk::put(&target, &user, &namespace_id, &path)
        }
        KvBulk::Delete { namespace, path } => {
            let (target, namespace_id) = target_and_namespace(namespace)?;
            commands::kv::bulk::delete(&target, &user, &namespace_id, &path)
        }
    }
}
