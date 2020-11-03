use std::{fs::File, io::Read, path::PathBuf};

use wasmparser::Validator;

use super::{config::V8_SUPPORTED_WASM_FEATURES, Lintable, Parseable};

#[derive(Debug)]
pub struct WebAssembly {
    bytes: Vec<u8>,
}

impl Parseable<(PathBuf, Option<PathBuf>)> for WebAssembly {
    fn parse(
        (binary_path, _text_path_opt): &(PathBuf, Option<PathBuf>),
    ) -> Result<Self, failure::Error> {
        let mut bytes: Vec<u8> = Vec::new();
        File::open(binary_path)?.read_to_end(&mut bytes)?;
        Ok(Self { bytes })
    }
}

impl Lintable for WebAssembly {
    fn lint(&self) -> Result<(), failure::Error> {
        let mut validator = Validator::new();
        validator.wasm_features(V8_SUPPORTED_WASM_FEATURES);

        match validator.validate_all(&self.bytes) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
