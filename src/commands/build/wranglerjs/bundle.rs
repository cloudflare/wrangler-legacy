use base64::decode;
#[cfg(test)]
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use crate::commands::build::wranglerjs::output::WranglerjsOutput;
#[cfg(test)]
use crate::terminal::message;

// Directory where we should write the {Bundle}. It represents the built
// artifact.
const BUNDLE_OUT: &str = "./worker";
pub struct Bundle {
    out: String,
}

// We call a {Bundle} the output of a {Bundler}; representing what {Webpack}
// produces.
impl Bundle {
    pub fn new() -> Bundle {
        Bundle {
            out: BUNDLE_OUT.to_string(),
        }
    }

    #[cfg(test)]
    fn new_at(out: String) -> Bundle {
        Bundle { out }
    }

    pub fn write(&self, wranglerjs_output: &WranglerjsOutput) -> Result<(), failure::Error> {
        let bundle_path = Path::new(&self.out);
        if !bundle_path.exists() {
            fs::create_dir(bundle_path)?;
        }

        let mut script_file = File::create(self.script_path())?;
        let mut script = create_prologue();
        script += &wranglerjs_output.script;

        if let Some(encoded_wasm) = &wranglerjs_output.wasm {
            let wasm = decode(encoded_wasm).expect("could not decode Wasm in base64");
            let mut wasm_file = File::create(self.wasm_path())?;
            wasm_file.write_all(&wasm)?;
        }

        script_file.write_all(script.as_bytes())?;

        Ok(())
    }

    pub fn wasm_path(&self) -> String {
        Path::new(&self.out)
            .join("module.wasm".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn has_wasm(&self) -> bool {
        Path::new(&self.wasm_path()).exists()
    }

    pub fn has_webpack_config(&self, webpack_config_path: &PathBuf) -> bool {
        webpack_config_path.exists()
    }

    pub fn get_wasm_binding(&self) -> String {
        "wasmprogram".to_string()
    }

    pub fn script_path(&self) -> String {
        Path::new(&self.out)
            .join("script.js".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }
}

// We inject some code at the top-level of the Worker; called {prologue}.
// This aims to provide additional support, for instance providing {window}.
pub fn create_prologue() -> String {
    r#"
        const window = this;
    "#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temp_dir(name: &str) -> String {
        let mut dir = env::temp_dir();
        dir.push(name);
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir(&dir).expect("could not create temp dir");

        dir.to_str().unwrap().to_string()
    }

    #[test]
    fn it_writes_the_bundle_script() {
        let out = create_temp_dir("it_writes_the_bundle_script");
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "foo".to_string(),
            wasm: None,
        };
        let bundle = Bundle::new_at(out.clone());

        bundle.write(&wranglerjs_output).unwrap();
        assert!(Path::new(&bundle.script_path()).exists());
        assert!(!Path::new(&bundle.wasm_path()).exists());

        cleanup(out);
    }

    #[test]
    fn it_writes_the_bundle_wasm() {
        let out = create_temp_dir("it_writes_the_bundle_wasm");
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "".to_string(),
            wasm: Some("abc".to_string()),
        };
        let bundle = Bundle::new_at(out.clone());

        bundle.write(&wranglerjs_output).unwrap();
        assert!(Path::new(&bundle.wasm_path()).exists());
        assert!(bundle.has_wasm());

        cleanup(out);
    }

    #[test]
    fn it_has_errors() {
        let wranglerjs_output = WranglerjsOutput {
            errors: vec!["a".to_string(), "b".to_string()],
            script: "".to_string(),
            wasm: None,
        };
        assert!(wranglerjs_output.has_errors());
        assert!(wranglerjs_output.get_errors() == "a\nb");
    }

    fn cleanup(name: String) {
        let current_dir = env::current_dir().unwrap();
        let path = Path::new(&current_dir).join(name);
        message::info(&format!("p: {:?}", path));
        fs::remove_dir_all(path).unwrap();
    }
}
