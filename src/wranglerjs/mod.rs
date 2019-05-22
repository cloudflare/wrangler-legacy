use crate::commands::publish::package::Package;
use serde::Deserialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

// FIXME(sven): make this private
#[derive(Deserialize, Debug)]
pub struct WrangerjsOutput {
    wasm: Option<String>,
    wasm_name: String,
    script: String,
    compiler_output: String,
}

impl WrangerjsOutput {
    pub fn compiler_output(&self) -> String {
        self.compiler_output.clone()
    }
}

const BUNDLE_OUT: &str = "./worker";
pub struct Bundle {}

impl Bundle {
    pub fn new() -> Bundle {
        Bundle {}
    }

    pub fn write(&self, wranglerjs_output: WrangerjsOutput) -> Result<(), failure::Error> {
        let mut metadata_file = File::create(self.metadata_path())?;
        metadata_file.write_all(create_metadata(self).as_bytes())?;

        let mut script_file = File::create(self.script_path())?;
        let mut script = create_prologue();

        match wranglerjs_output.wasm {
            Some(wasm) => {
                let mut wasm_file = File::create(self.wasm_path())?;
                wasm_file.write_all(wasm.as_bytes())?;

                script +=
                    &create_wasm_prologue(wranglerjs_output.wasm_name, self.get_wasm_binding());
            }
            None => {}
        }

        script += &wranglerjs_output.script;
        script_file.write_all(script.as_bytes())?;

        Ok(())
    }

    pub fn metadata_path(&self) -> String {
        Path::new(BUNDLE_OUT)
            .join("metadata.json".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn wasm_path(&self) -> String {
        Path::new(BUNDLE_OUT)
            .join("module.wasm".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn has_wasm(&self) -> bool {
        Path::new(&self.wasm_path()).exists()
    }

    pub fn has_webpack_config(&self) -> bool {
        Path::new("webpack.config.js").exists()
    }

    pub fn get_wasm_binding(&self) -> String {
        assert!(self.has_wasm());
        "wasmprogram".to_string()
    }

    pub fn script_path(&self) -> String {
        Path::new(BUNDLE_OUT)
            .join("script.js".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }
}

fn executable_path() -> PathBuf {
    Path::new(".")
        .join("node_modules")
        .join(".bin")
        .join("wrangler-js")
}

pub fn run_build(
    wasm_pack_path: PathBuf,
    bundle: &Bundle,
) -> Result<WrangerjsOutput, failure::Error> {
    if !Path::new(BUNDLE_OUT).exists() {
        fs::create_dir(BUNDLE_OUT)?;
    }

    let mut command = Command::new(executable_path());
    command.env("WASM_PACK_PATH", wasm_pack_path);

    // if {webpack.config.js} is not present, we infer the entry based on the
    // {package.json} file and pass it to {wrangler-js}.
    // https://github.com/cloudflare/wrangler/issues/98
    if !bundle.has_webpack_config() {
        let package = Package::new("./")?;
        let current_dir = env::current_dir()?;
        let package_main = current_dir.join(package.main).to_str().unwrap().to_string();
        command.arg("--no-webpack-config=1");
        command.arg(format!("--use-entry={}", package_main));
    }

    let output = command.output().expect("failed to execute process");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
    // println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout))
            .expect("could not parse wranglerjs output"),
    )
}

pub fn run_npm_install() -> Result<(), failure::Error> {
    let mut command = Command::new("npm");
    command.arg("install");
    let output = command.output().expect("failed to execute process");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
    Ok(())
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
    r#"
        const window = this;
    "#
    .to_string()
}

pub fn create_wasm_prologue(name: String, binding: String) -> String {
    format!(
        r#"
            const oldFetch = fetch;
            function fetch(name) {{
              if (name === "{name}") {{
                return Promise.resolve({{
                  arrayBuffer() {{
                    return {binding}; // defined in bindinds
                  }}
                }});
              }}
              return oldFetch(name);
            }}
        "#,
        name = name,
        binding = binding
    )
    .to_string()
}

fn create_metadata(bundle: &Bundle) -> String {
    if bundle.has_wasm() {
        format!(
            r#"
                {{
                    "body_part": "script",
                    "binding": {{
                        "name": "{name}",
                        "type": "wasm_module",
                        "part": "{name}"
                    }}
                }}
            "#,
            name = bundle.get_wasm_binding(),
        )
        .to_string()
    } else {
        format!(
            r#"
                {{
                    "body_part": "script"
                }}
            "#
        )
        .to_string()
    }
}
