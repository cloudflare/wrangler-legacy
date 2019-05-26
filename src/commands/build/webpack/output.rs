use serde::Deserialize;

// This structure represents the communication between {wrangler-js} and
// {wrangler}. It is send back after {wrangler-js} completion.
// FIXME(sven): make this private
#[derive(Deserialize, Debug)]
pub struct WranglerjsOutput {
    pub wasm: Option<String>,
    pub script: String,
    // {wrangler-js} will send us the path to the {dist} directory that {Webpack}
    // used; it's tedious to remove a directory with content in JavaScript so
    // let's do it in Rust!
    pub dist_to_clean: Option<String>,
}
