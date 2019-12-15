use super::cli_prelude::*;

/// Defines the `generate` sub-command entry point.
pub fn sub_command() -> App {
    #[cfg(target_os = "macos")]
    const ABOUT: &str = "👯  Generate a new worker project";
    #[cfg(not(target_os = "macos"))]
    const ABOUT: &str = "Generate a new worker project";

    SubCommand::with_name("generate")
        .about(ABOUT)
        .arg(
            Arg::with_name("name")
                .help("the name of your worker! defaults to 'worker'")
                .index(1),
        )
        .arg(
            Arg::with_name("template")
                .help("a link to a github template! defaults to https://github.com/cloudflare/worker-template")
                .index(2),
        )
        .arg(
            Arg::with_name("type")
                .short("t")
                .long("type")
                .takes_value(true)
                .help("the type of project you want generated"),
        )
        .arg(
            Arg::with_name("site")
                .short("s")
                .long("site")
                .takes_value(false)
                .help("initializes a Workers Sites project. Overrides `type` and `template`"),
        )
}
