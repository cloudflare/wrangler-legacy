use flate2::write::ZlibEncoder;
use flate2::Compression;
use number_prefix::{NumberPrefix, Prefixed, Standalone};
use serde::Deserialize;
use std::io::prelude::*;

// This structure represents the communication between {wranglerjs} and
// {wrangler}. It is send back after {wranglerjs} completion.
// FIXME(sven): make this private
#[derive(Deserialize, Debug)]
pub struct WranglerjsOutput {
    pub wasm: Option<String>,
    pub script: String,
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

    pub fn script_size(&self) -> String {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(&self.script.as_bytes())
            .expect("could not write buffer");
        let compressed_bytes = e.finish();

        match NumberPrefix::decimal(compressed_bytes.unwrap().len() as f64) {
            Standalone(bytes) => format!("{} bytes", bytes),
            Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
        }
    }

    pub fn wasm_size(&self) -> String {
        let size = self.wasm.to_owned().unwrap().len();
        match NumberPrefix::decimal(size as f64) {
            Standalone(bytes) => format!("{} bytes", bytes),
            Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_gzip_script_size() {
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "aaaa".to_string(),
            wasm: None,
        };

        assert_eq!(wranglerjs_output.script_size(), "12 bytes");
    }

    #[test]
    fn it_returns_wasm_size() {
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "".to_string(),
            wasm: Some("abc".to_string()),
        };

        assert_eq!(wranglerjs_output.wasm_size(), "3 bytes");
    }
}
