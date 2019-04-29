use std::process::Command;
use std::path::Path;

pub struct Bundle {}

impl Bundle {
    pub fn new() -> Bundle {
        Bundle {}
    }

    pub fn metadata_path(&self) -> String {
        "./worker/metadata.json".to_string()
    }

    pub fn wasm_path(&self) -> String {
        "./worker/module.wasm".to_string()
    }

    pub fn has_wasm(&self) -> bool {
        Path::new(&self.wasm_path()).exists()
    }

    pub fn script_path(&self) -> String {
        "./worker/script.js".to_string()
    }
}

pub fn run_build() -> Result<(), failure::Error> {
    let output = Command::new("wrangler-js")
        .output()
        .expect("failed to execute process");
    assert!(output.status.success());
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
