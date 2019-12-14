// Third-Party Imports.
pub use clap::AppSettings;
pub use clap::ArgMatches;

// Type aliases for `clap` structures.
pub type App = clap::App<'static, 'static>;
pub type Arg = clap::Arg<'static, 'static>;
pub type ArgGroup = clap::ArgGroup<'static>;
pub type SubCommand = clap::SubCommand<'static>;
