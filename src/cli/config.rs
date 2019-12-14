use super::cli_prelude::*;

pub fn sub_command() -> App {
    SubCommand::with_name("config")
        .about( "Setup wrangler with your Cloudflare account")
        .arg(
            Arg::with_name("api-key")
                .help("use an email and global API key for authentication. This is not recommended; use API tokens (the default) if possible")
                .long("api-key")
                .takes_value(false),
        ).arg(
            Arg::with_name("no-verify")
                .help("do not verify provided credentials before writing out Wrangler config file")
                .long("no-verify")
                .takes_value(false)
        )
}
