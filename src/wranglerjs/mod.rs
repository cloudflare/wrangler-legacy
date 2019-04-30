use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

// FIXME(sven): make this private
#[derive(Deserialize, Debug)]
pub struct WrangerjsOutput {
    pub wasm: Option<String>,
    pub wasm_name: String,
    pub script: String,
}

pub struct Bundle {}

impl Bundle {
    pub fn new() -> Bundle {
        Bundle {}
    }

    pub fn write(&self, wranglerjs_output: WrangerjsOutput) -> Result<(), failure::Error> {
        let mut metadata_file = File::create(self.metadata_path())?;
        metadata_file.write_all(create_metadata(self).as_bytes())?;

        let mut script_file = File::create(self.script_path())?;
        let mut script = wranglerjs_output.script;
        script += &create_prologue();

        match wranglerjs_output.wasm {
            Some(wasm) => {
                let mut wasm_file = File::create(self.wasm_path())?;
                wasm_file.write_all(wasm.as_bytes())?;

                script +=
                    &create_wasm_prologue(wranglerjs_output.wasm_name, self.get_wasm_binding());
            }
            None => {}
        }

        script_file.write_all(script.as_bytes())?;

        Ok(())
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

    pub fn get_wasm_binding(&self) -> String {
        assert!(self.has_wasm());
        "wasmprogram".to_string()
    }

    pub fn script_path(&self) -> String {
        "./worker/script.js".to_string()
    }
}

fn executable_path() -> PathBuf {
    Path::new(".")
        .join("node_modules")
        .join(".bin")
        .join("wrangler-js")
}

pub fn run_build() -> Result<WrangerjsOutput, failure::Error> {
    let output = Command::new(executable_path())
        .output()
        .expect("failed to execute process");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
            .expect("could not parse wranglerjs output"),
    )
}

pub fn is_installed() -> bool {
    executable_path().exists()
}

pub fn install() -> Result<(), failure::Error> {
    let output = Command::new("npm")
        .arg("install")
        .arg("wrangler-js")
        .output()
        .expect("failed to execute process");
    assert!(output.status.success());
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

pub fn create_prologue() -> String {
    "
        const window = this;
    "
    .to_string()
}

pub fn create_wasm_prologue(name: String, binding: String) -> String {
    format!(
        "
        function fetch(name) {{
          if (name === \"{name}\") {{
            return Promise.resolve({{
              arrayBuffer() {{
                return {binding}; // defined in bindinds
              }}
            }});
          }}
          throw new Error(\"unreachable: attempt to fetch \" + name);
        }}
    ",
        name = name,
        binding = binding
    )
    .to_string()
}

fn create_metadata(bundle: &Bundle) -> String {
    if bundle.has_wasm() {
        format!(
            "
                {{
                    \"body_part\": \"script\",
                    \"binding\": {{
                        \"name\": \"{name}\",
                        \"type\": \"wasm_module\",
                        \"part\": \"{name}\"
                    }}
                }}
            ",
            name = bundle.get_wasm_binding(),
        )
        .to_string()
    } else {
        format!(
            "
                {{
                    \"body_part\": \"script\"
                }}
            "
        )
        .to_string()
    }
}
