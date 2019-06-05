#[cfg(test)]
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use log::info;

use crate::commands::build::wranglerjs::output::WranglerjsOutput;

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

    pub fn write(&self, wranglerjs_output: WranglerjsOutput) -> Result<(), failure::Error> {
        let bundle_path = Path::new(&self.out);
        if !bundle_path.exists() {
            fs::create_dir(bundle_path)?;
        }

        let mut script_file = File::create(self.script_path())?;
        let mut script = create_prologue();
        script += &wranglerjs_output.script;

        if let Some(wasm) = wranglerjs_output.wasm {
            let mut wasm_file = File::create(self.wasm_path())?;
            wasm_file.write_all(wasm.as_bytes())?;
        }

        script_file.write_all(script.as_bytes())?;

        let mut metadata_file = File::create(self.metadata_path())?;
        metadata_file.write_all(create_metadata(self).as_bytes())?;

        // cleanup {Webpack} dist, if specified.
        if let Some(dist_to_clean) = wranglerjs_output.dist_to_clean {
            info!("Remove {}", dist_to_clean);
            fs::remove_dir_all(dist_to_clean).expect("could not clean Webpack dist.");
        }

        Ok(())
    }

    pub fn metadata_path(&self) -> String {
        Path::new(&self.out)
            .join("metadata.json".to_string())
            .to_str()
            .unwrap()
            .to_string()
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

    pub fn has_webpack_config(&self) -> bool {
        Path::new("webpack.config.js").exists()
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

// This metadata describe the bindings on the Worker.
fn create_metadata(bundle: &Bundle) -> String {
    info!("create metadata; wasm={}", bundle.has_wasm());
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
        r#"
            {
                "body_part": "script"
            }
        "#
        .to_string()
    }
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
    fn it_writes_the_bundle_metadata() {
        let out = create_temp_dir("it_writes_the_bundle_metadata");
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "".to_string(),
            dist_to_clean: None,
            wasm: None,
        };
        let bundle = Bundle::new_at(out.clone());

        bundle.write(wranglerjs_output).unwrap();
        assert!(Path::new(&bundle.metadata_path()).exists());
        let contents =
            fs::read_to_string(&bundle.metadata_path()).expect("could not read metadata");

        assert_eq!(
            contents,
            r#"
            {
                "body_part": "script"
            }
        "#
        );

        cleanup(out);
    }

    #[test]
    fn it_writes_the_bundle_script() {
        let out = create_temp_dir("it_writes_the_bundle_script");
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "foo".to_string(),
            dist_to_clean: None,
            wasm: None,
        };
        let bundle = Bundle::new_at(out.clone());

        bundle.write(wranglerjs_output).unwrap();
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
            dist_to_clean: None,
        };
        let bundle = Bundle::new_at(out.clone());

        bundle.write(wranglerjs_output).unwrap();
        assert!(Path::new(&bundle.wasm_path()).exists());
        assert!(bundle.has_wasm());

        cleanup(out);
    }

    #[test]
    fn it_writes_the_bundle_wasm_metadata() {
        let out = create_temp_dir("it_writes_the_bundle_wasm_metadata");
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "".to_string(),
            wasm: Some("abc".to_string()),
            dist_to_clean: None,
        };
        let bundle = Bundle::new_at(out.clone());

        bundle.write(wranglerjs_output).unwrap();
        assert!(Path::new(&bundle.metadata_path()).exists());
        let contents =
            fs::read_to_string(&bundle.metadata_path()).expect("could not read metadata");

        assert_eq!(
            contents,
            r#"
                {
                    "body_part": "script",
                    "binding": {
                        "name": "wasmprogram",
                        "type": "wasm_module",
                        "part": "wasmprogram"
                    }
                }
            "#
        );

        cleanup(out);
    }

    #[test]
    fn it_has_errors() {
        let wranglerjs_output = WranglerjsOutput {
            errors: vec!["a".to_string(), "b".to_string()],
            script: "".to_string(),
            wasm: None,
            dist_to_clean: None,
        };
        assert!(wranglerjs_output.has_errors());
        assert!(wranglerjs_output.get_errors() == "a\nb");
    }

    fn cleanup(name: String) {
        let current_dir = env::current_dir().unwrap();
        let path = Path::new(&current_dir).join(name);
        println!("p: {:?}", path);
        fs::remove_dir_all(path).unwrap();
    }
}
