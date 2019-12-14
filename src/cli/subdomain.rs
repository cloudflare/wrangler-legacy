use super::cli_prelude::*;

pub fn sub_command() -> App {
    SubCommand::with_name("subdomain")
        .about("Configure your workers.dev subdomain")
        .arg(
            Arg::with_name("name")
                .help("the subdomain on workers.dev you'd like to reserve")
                .index(1),
        )
}
