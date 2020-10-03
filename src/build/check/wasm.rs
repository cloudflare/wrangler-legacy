use std::path::PathBuf;

use wasmparser::{Data, WasmFeatures};
use wast::Module;

use super::{Lintable, Parseable, Validate};

pub struct WebAssembly {
    binary: Data<'static>,
    text: Option<Module<'static>>,
}

impl Parseable<(PathBuf, Option<PathBuf>)> for WebAssembly {
    fn parse(input: &(PathBuf, Option<PathBuf>)) -> Result<Self, failure::Error> {
        todo!()
    }
}

impl Lintable<WasmFeatures> for WebAssembly {
    fn lint(&self, args: WasmFeatures) -> Result<(), failure::Error> {
        todo!()
    }
}

impl Validate<WasmFeatures, (PathBuf, Option<PathBuf>)> for WebAssembly {
    fn validate(&self) -> Result<(), failure::Error> {
        todo!()
    }
}
