use std::{fs::File, io::Read, path::PathBuf};

use wasmparser::Validator;

use super::{config::V8_SUPPORTED_WASM_FEATURES, Lintable, Parseable, Validate};

#[derive(Debug)]
pub struct WebAssembly;

impl Parseable<(PathBuf, Option<PathBuf>)> for WebAssembly {
    fn parse(
        (_binary_path, _text_path_opt): &(PathBuf, Option<PathBuf>),
    ) -> Result<Self, failure::Error> {
        unimplemented!()
    }
}

impl Lintable for WebAssembly {
    fn lint(&self) -> Result<(), failure::Error> {
        unimplemented!()
    }
}

impl Validate<(PathBuf, Option<PathBuf>)> for WebAssembly {
    fn validate((binary_path, _): (PathBuf, Option<PathBuf>)) -> Result<(), failure::Error> {
        let mut validator = Validator::new();
        let mut bytes: Vec<u8> = Vec::new();

        File::open(binary_path)?.read_to_end(&mut bytes)?;
        validator.wasm_features(V8_SUPPORTED_WASM_FEATURES);

        match validator.validate_all(&bytes) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
