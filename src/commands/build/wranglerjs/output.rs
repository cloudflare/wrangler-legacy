use serde::Deserialize;

// This structure represents the communication between {wranglerjs} and
// {wrangler}. It is send back after {wranglerjs} completion.
// FIXME(sven): make this private
#[derive(Deserialize, Debug)]
pub struct WranglerjsOutput {
    pub wasm: Option<String>,
    pub script: String,
    // {wranglerjs} will send us the path to the {dist} directory that {Webpack}
    // used; it's tedious to remove a directory with content in JavaScript so
    // let's do it in Rust!
    pub dist_to_clean: Option<String>,
    // Errors emited by {wranglerjs}, if any
    pub errors: Vec<String>,
}

impl WranglerjsOutput {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> String {
        self.errors.join("\n")
    }
}
