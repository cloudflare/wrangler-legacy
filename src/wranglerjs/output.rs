use crate::terminal::emoji;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use number_prefix::NumberPrefix;
use serde::Deserialize;
use std::io::prelude::*;

// This structure represents the communication between {wranglerjs} and
// {wrangler}. It is sent back after {wranglerjs} completion.
// TODO: (sven) make this private
#[derive(Deserialize, Debug)]
pub struct WranglerjsOutput {
    pub wasm: Option<String>,
    pub script: String,
    // Errors emitted by {wranglerjs}, if any
    pub errors: Vec<String>,
}

impl WranglerjsOutput {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> String {
        self.errors.join("\n")
    }

    fn project_size_bytes(&self) -> u64 {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

        // approximation of how projects are gzipped
        e.write_all(self.script.as_bytes())
            .expect("could not write script buffer");

        if let Some(wasm) = &self.wasm {
            e.write_all(wasm.to_owned().as_bytes())
                .expect("could not write wasm buffer");
        }

        e.finish().expect("failed to compress project").len() as u64
    }

    fn project_size_message(compressed_size: u64) -> String {
        const MAX_PROJECT_SIZE: u64 = 1 << 20; // 1 MiB
        const WARN_THRESHOLD: u64 = MAX_PROJECT_SIZE - 81_920; // Warn when less than 80 KiB left to grow, ~92% usage
        const MAX_BEFORE_WARN: u64 = WARN_THRESHOLD - 1;

        let bytes_left = MAX_PROJECT_SIZE.checked_sub(compressed_size);

        let human_size = match NumberPrefix::binary(compressed_size as f64) {
            NumberPrefix::Standalone(bytes) => format!("{} bytes", bytes),
            NumberPrefix::Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
        };

        let human_leftover = if let Some(bytes_left) = bytes_left {
            let msg = match NumberPrefix::binary(bytes_left as f64) {
                NumberPrefix::Standalone(bytes) => format!("{} bytes", bytes),
                NumberPrefix::Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
            };
            Some(msg)
        } else {
            None
        };

        match compressed_size {
            WARN_THRESHOLD..=MAX_PROJECT_SIZE => format!("{}. {2} Your built project is {} away from reaching the 1MiB size limit. {2}", human_size, human_leftover.expect("failed to get leftover bytes"), emoji::WARN),
            0..=MAX_BEFORE_WARN => format!("{}.", human_size),
            _ => format!("{}. {1} Your built project has grown past the 1MiB size limit and may fail to deploy. {1}", human_size, emoji::WARN)
        }
    }

    pub fn project_size(&self) -> String {
        Self::project_size_message(self.project_size_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_warns_over_max_size() {
        assert!(WranglerjsOutput::project_size_message(1 << 21).contains("grown past"));
    }

    #[test]
    fn it_warns_near_max_size() {
        assert!(WranglerjsOutput::project_size_message((1 << 20) - 4096).contains("reaching"));
    }

    #[test]
    fn it_returns_project_size_with_wasm() {
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "abcdefg".to_string(),
            wasm: Some("123456".to_string()),
        };

        assert_eq!(wranglerjs_output.project_size_bytes(), 21);
    }

    #[test]
    fn it_returns_project_size_without_wasm() {
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "abcdefg".to_string(),
            wasm: None,
        };

        assert_eq!(wranglerjs_output.project_size_bytes(), 15);
    }
}
