use std::{fmt::Debug, path::Path};

pub fn check_wasm<P: AsRef<Path> + Debug>(_path: P) -> Result<String, failure::Error> {
    todo!()
}
