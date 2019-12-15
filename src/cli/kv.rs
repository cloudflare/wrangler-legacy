use super::cli_prelude::*;

/// Defines the `kv::namespace` sub-command entry point.
fn kv_namespace_subcommand(args: &KvSubcommandArgs, sub_commands: &mut Vec<App>) {
    #[cfg(target_os = "macos")]
    const ABOUT: &str = "🕵️  Interact with your Workers KV Namespaces";
    #[cfg(not(target_os = "macos"))]
    const ABOUT: &str = "Interact with your Workers KV Namespaces";

    let cmd = SubCommand::with_name("kv:namespace")
        .about(ABOUT)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("create")
                .about("Create a new namespace")
                .arg(&args.env)
                .arg(
                    Arg::with_name("binding")
                        .help("The binding for your new namespace")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Delete namespace")
                .arg(&args.kv_binding)
                .arg(&args.kv_namespace_id)
                .arg(&args.env)
                .group(KvSubcommandArgs::arg_group()),
        )
        .subcommand(
            SubCommand::with_name("list").about("List all namespaces on your Cloudflare account"),
        );

    sub_commands.push(cmd);
}

/// Defines the `kv::key` sub-command entry point.
fn kv_key_subcommand(args: &KvSubcommandArgs, sub_commands: &mut Vec<App>) {
    #[cfg(target_os = "macos")]
    const ABOUT: &str = "🔑  Individually manage Workers KV key-value pairs";
    #[cfg(not(target_os = "macos"))]
    const ABOUT: &str = "Individually manage Workers KV key-value pairs";

    let cmd = SubCommand::with_name("kv:key")
        .about(ABOUT)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("put")
                .about("Put a key-value pair into a namespace")
                .arg(&args.kv_binding)
                .arg(&args.kv_namespace_id)
                .arg(&args.env)
                .group(KvSubcommandArgs::arg_group())
                .arg(
                    Arg::with_name("key")
                    .help("Key to write value to")
                    .required(true)
                    .index(1)
                )
                .arg(
                    Arg::with_name("value")
                    .help("Value for key")
                    .required(true)
                    .index(2)
                )
                .arg(
                    Arg::with_name("expiration-ttl")
                    .help("Number of seconds for which the entries should be visible before they expire. At least 60. Takes precedence over 'expiration' option")
                    .short("t")
                    .long("ttl")
                    .value_name("SECONDS")
                    .takes_value(true)
                )
                .arg(
                    Arg::with_name("expiration")
                    .help("Number of seconds since the UNIX epoch, indicating when the key-value pair should expire")
                    .short("x")
                    .long("expiration")
                    .takes_value(true)
                    .value_name("SECONDS")
                )
                .arg(
                    Arg::with_name("path")
                    .help("The value passed in is a path to a file; open and upload its contents")
                    .short("p")
                    .long("path")
                    .takes_value(false)
                )
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get a key's value from a namespace")
                .arg(&args.kv_binding)
                .arg(&args.kv_namespace_id)
                .arg(&args.env)
                .group(KvSubcommandArgs::arg_group())
                .arg(
                    Arg::with_name("key")
                    .help("Key whose value to get")
                    .required(true)
                    .index(1)
                )
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Delete a key and its value from a namespace")
                .arg(&args.kv_binding)
                .arg(&args.kv_namespace_id)
                .arg(&args.env)
                .group(KvSubcommandArgs::arg_group())
                .arg(
                    Arg::with_name("key")
                    .help("Key whose value to delete")
                    .required(true)
                    .index(1)
                )
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List all keys in a namespace. Produces JSON output")
                .arg(&args.kv_binding)
                .arg(&args.kv_namespace_id)
                .arg(&args.env)
                .group(KvSubcommandArgs::arg_group())
                .arg(
                    Arg::with_name("prefix")
                    .help("The prefix for filtering listed keys")
                    .short("p")
                    .long("prefix")
                    .value_name("STRING")
                    .takes_value(true),
                )
    );

    sub_commands.push(cmd);
}

/// Defines the `kv::bulk` sub-command entry point.
fn kv_bulk_subcommand(args: &KvSubcommandArgs, sub_commands: &mut Vec<App>) {
    #[cfg(target_os = "macos")]
    const ABOUT: &str = "💪  Interact with multiple Workers KV key-value pairs at once";
    #[cfg(not(target_os = "macos"))]
    const ABOUT: &str = "Interact with multiple Workers KV key-value pairs at once";

    let cmd = SubCommand::with_name("kv:bulk")
        .about(ABOUT)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("put")
                .about("Upload multiple key-value pairs to a namespace")
                .arg(&args.kv_binding)
                .arg(&args.kv_namespace_id)
                .arg(&args.env)
                .group(KvSubcommandArgs::arg_group())
                .arg(
                    Arg::with_name("path")
                    .help("the JSON file of key-value pairs to upload, in form [{\"key\":..., \"value\":...}\"...]")
                    .required(true)
                    .index(1)
                )
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Delete multiple keys and their values from a namespace")
                .arg(&args.kv_binding)
                .arg(&args.kv_namespace_id)
                .arg(&args.env)
                .group(KvSubcommandArgs::arg_group())
                        .arg(
                            Arg::with_name("path")
                            .help("the JSON file of key-value pairs to upload, in form [\"<example-key>\", ...]")
                            .required(true)
                            .index(1)
                        )
        );

    sub_commands.push(cmd);
}

/// Aggregates all kv sub-commands into a `Vec<App>`.
/// This `Vec<App>` is used to register each of the kv sub-commands within the
/// kv-command-suite.
pub fn all_kv_sub_commands() -> Vec<App> {
    let mut all_kv_sub_cmds: Vec<App> = vec![];
    let kv_sub_args = KvSubcommandArgs::default();

    kv_namespace_subcommand(&kv_sub_args, &mut all_kv_sub_cmds);
    kv_key_subcommand(&kv_sub_args, &mut all_kv_sub_cmds);
    kv_bulk_subcommand(&kv_sub_args, &mut all_kv_sub_cmds);

    all_kv_sub_cmds
}

/// Structures together arguments common to all `kv` commands.
/// This structure represents 3 of 4 arguments `kv` commands require.
/// The fourth argument is provided via an associated function, `arg_group`.
struct KvSubcommandArgs {
    kv_binding: Arg,
    kv_namespace_id: Arg,
    env: Arg,
}

impl KvSubcommandArgs {
    /// Returns the appropriate argument group for use within kv sub-sub-commands.
    /// Each `clap::SubCommand` supports a `group` method. This `group` method takes
    /// ownership of its `ArgGroup` argument: (e.g. pub fn group(mut self, group: ArgGroup<'a>))`.
    /// So, this approach seems reasonable even though it does also seem redundant.
    fn arg_group() -> ArgGroup {
        ArgGroup::with_name("namespace-specifier")
            .args(&["binding", "namespace-id"])
            .required(true)
    }
}

impl<'a> Default for KvSubcommandArgs {
    fn default() -> Self {
        let kv_binding = Arg::with_name("binding")
            .help("The binding of the namespace this action applies to")
            .short("b")
            .long("binding")
            .value_name("BINDING NAME")
            .takes_value(true);

        let kv_namespace_id = Arg::with_name("namespace-id")
            .help("The id of the namespace this action applies to")
            .short("n")
            .long("namespace-id")
            .value_name("ID")
            .takes_value(true);

        let env = Arg::with_name("env")
            .help("Environment to use")
            .short("e")
            .long("env")
            .takes_value(true)
            .value_name("ENVIRONMENT NAME");

        KvSubcommandArgs {
            kv_binding,
            kv_namespace_id,
            env,
        }
    }
}
