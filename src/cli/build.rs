use super::cli_prelude::*;

pub fn sub_command() -> App {
    // TODO: <high-priority> bring back emoji::Crab!
    SubCommand::with_name("build")
        .about("Build your worker")
        .arg(
            Arg::with_name("env")
                .help("environment to build")
                .short("e")
                .long("env")
                .takes_value(true),
        )
}
