use super::cli_prelude::*;

/// Defines the `preview` sub-command entry point.
pub fn sub_command() -> App {
    #[cfg(target_os = "macos")]
    const ABOUT: &str = "🔬  Preview your code temporarily on cloudflareworkers.com";
    #[cfg(not(target_os = "macos"))]
    const ABOUT: &str = "Preview your code temporarily on cloudflareworkers.com";

    SubCommand::with_name("preview")
        .about(ABOUT)
        .arg(
            Arg::with_name("headless")
                .help("Don't open the browser on preview")
                .long("headless")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("method")
                .help("Type of request to preview your worker with (get, post)")
                .index(1),
        )
        .arg(
            Arg::with_name("body")
                .help("Body string to post to your preview worker request")
                .index(2),
        )
        .arg(
            Arg::with_name("env")
                .help("environment to preview")
                .short("e")
                .long("env")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("watch")
                .help("watch your project for changes and update the preview automagically")
                .long("watch")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .takes_value(false)
                .help("toggle verbose output"),
        )
}
