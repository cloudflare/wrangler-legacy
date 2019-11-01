use clap::{App, Arg, SubCommand};

use crate::terminal::emoji;

pub fn get() -> App {
    SubCommand::with_name("init")
        .about(&*format!(
            "{} Create a wrangler.toml for an existing project",
            emoji::INBOX
        ))
        .arg(
            Arg::with_name("name")
                .help("the name of your worker! defaults to 'worker'")
                .index(1),
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
