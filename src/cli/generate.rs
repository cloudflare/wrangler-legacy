use super::cli_prelude::*;

pub fn sub_command() -> App {
    SubCommand::with_name("generate")
        .about("Generate a new worker project")
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
