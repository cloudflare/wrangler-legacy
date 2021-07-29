use anyhow::Result;
use base64::decode;

#[cfg(test)]
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use super::output::WranglerjsOutput;

// Directory where we should write the {Bundle}. It represents the built
// artifact.
const BUNDLE_OUT: &str = "worker";
pub struct Bundle {
    out: PathBuf,
}

// We call a {Bundle} the output of a {Bundler}; representing what {Webpack}
// produces.
impl Bundle {
    pub fn new(package_dir: &Path) -> Bundle {
        Bundle {
            out: package_dir.join(BUNDLE_OUT),
        }
    }

    #[cfg(test)]
    fn new_at(out: PathBuf) -> Bundle {
        Bundle { out }
    }

    pub fn write(&self, wranglerjs_output: &WranglerjsOutput) -> Result<()> {
        if !self.out.exists() {
            fs::create_dir(&self.out)?;
        }

        let mut script_file = File::create(self.script_path())?;

        if let Some(encoded_wasm) = &wranglerjs_output.wasm {
            let wasm = decode(encoded_wasm).expect("could not decode Wasm in base64");
            let mut wasm_file = File::create(self.wasm_path())?;
            wasm_file.write_all(&wasm)?;
        }

        if self.has_wasm() {
            script_file.write_all(
                format!(
                    r#"
                        WebAssembly.instantiateStreaming =
                            async function instantiateStreaming(req, importObject) {{
                          const module = {};
                          return {{
                            module,
                            instance: new WebAssembly.Instance(module, importObject)
                          }}
                        }};
                    "#,
                    self.get_wasm_binding()
                )
                .as_bytes(),
            )?;
        }
        script_file.write_all(wranglerjs_output.script.as_bytes())?;

        Ok(())
    }

    pub fn wasm_path(&self) -> PathBuf {
        PathBuf::from(&self.out).join("module.wasm")
    }

    pub fn has_wasm(&self) -> bool {
        self.wasm_path().exists()
    }

    pub fn get_wasm_binding(&self) -> String {
        "WASM_MODULE".to_string()
    }

    pub fn script_path(&self) -> PathBuf {
        PathBuf::from(&self.out).join("script.js")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn create_temp_dir(name: &str) -> PathBuf {
        let mut dir = env::temp_dir();
        dir.push(name);
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir(&dir).expect("could not create temp dir");

        dir
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

    fn cleanup(name: PathBuf) {
        let current_dir = env::current_dir().unwrap();
        let path = Path::new(&current_dir).join(name);
        fs::remove_dir_all(path).unwrap();
    }
}
