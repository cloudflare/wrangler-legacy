use base64::decode;
use serde::Deserialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

use log::info;

use crate::settings::binding::Binding;
#[cfg(test)]
use crate::terminal::message;
use crate::worker_bundle::WorkerBundle;

pub const WASM_BINDING: &str = "wasmprogram";

// This structure represents the communication between {wranglerjs} and
// {wrangler}. It is send back after {wranglerjs} completion.
// FIXME(sven): make this private
#[derive(Deserialize, Debug)]
pub struct WranglerjsOutput {
    pub wasm: Option<String>,
    pub script: String,
    // {wranglerjs} will send us the path to the {dist} directory that {Webpack}
    // used; it's tedious to remove a directory with content in JavaScript so
    // let's do it in Rust!
    pub dist_to_clean: Option<String>,
    // Errors emited by {wranglerjs}, if any
    pub errors: Vec<String>,
}

impl WranglerjsOutput {
    pub fn to_worker_bundle_in(
        &self,
        out_root: Option<PathBuf>,
    ) -> Result<WorkerBundle, failure::Error> {
        // `wranglerjs` sends us the file content directly, in order to construct
        // the WorkerBundle we need to persist them somewhere.
        let dist = env::temp_dir().join("wrangler_webpack");

        let mut bindings = vec![];
        let script_path = Path::new(&dist).join("script.js".to_string());

        let bundle_path = Path::new(&dist);
        if !bundle_path.exists() {
            fs::create_dir(bundle_path)?;
        }

        let mut script_file = File::create(&script_path)?;
        let mut script = create_prologue();
        script += &self.script;

        if let Some(encoded_wasm) = &self.wasm {
            let path = Path::new(&dist).join("module.wasm".to_string());

            let wasm = decode(encoded_wasm).expect("could not decode Wasm in base64");
            let mut wasm_file = File::create(&path)?;
            wasm_file.write_all(&wasm)?;

            bindings.push(Binding::new_wasm_module(
                path,                     // path
                WASM_BINDING.to_string(), // name
                WASM_BINDING.to_string(), // part
            ));
        };

        script_file.write_all(script.as_bytes())?;

        // cleanup {Webpack} dist, if specified.
        if let Some(dist_to_clean) = &self.dist_to_clean {
            info!("Remove {}", dist_to_clean);
            fs::remove_dir_all(dist_to_clean).expect("could not clean Webpack dist.");
        }

        Ok(WorkerBundle {
            script_path,
            bindings,

            // let the WorkerBundle generate the metadata file based on the bindings
            metadata_path: None,
            out_root,
        })
    }

    pub fn to_worker_bundle(&self) -> Result<WorkerBundle, failure::Error> {
        self.to_worker_bundle_in(None)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> String {
        self.errors.join("\n")
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
    fn it_writes_the_bundle_metadata() {
        let out = create_temp_dir("it_writes_the_bundle_metadata");
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "".to_string(),
            dist_to_clean: None,
            wasm: None,
        };
        let bundle = wranglerjs_output
            .to_worker_bundle_in(Some(out.clone()))
            .unwrap();
        bundle.persist().unwrap();

        assert!(out.join("metadata.json").exists());
        let contents =
            fs::read_to_string(out.join("metadata.json")).expect("could not read metadata");

        assert_eq!(contents, r#"{"body_part":"script","bindings":[]}"#);

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
        let bundle = wranglerjs_output
            .to_worker_bundle_in(Some(out.clone()))
            .expect("could not create WorkerBundle");

        assert!(bundle.script_path.exists());
        assert!(!bundle.has_wasm());

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
        let bundle = wranglerjs_output
            .to_worker_bundle_in(Some(out.clone()))
            .expect("could not create WorkerBundle");

        assert!(bundle.has_wasm());
        cleanup(out);
    }

    #[test]
    fn it_writes_the_bundle_wasm_metadata() {
        let out = create_temp_dir("it_writes_the_bundle_wasm_metadata");
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "".to_string(),
            wasm: Some("YWJjCg==".to_string()),
            dist_to_clean: None,
        };
        let bundle = wranglerjs_output
            .to_worker_bundle_in(Some(out.clone()))
            .expect("could not create WorkerBundle");
        bundle.persist().unwrap();

        assert!(out.join("metadata.json").exists());
        let contents =
            fs::read_to_string(&out.join("metadata.json")).expect("could not read metadata");

        assert_eq!(
            contents,
            r#"{"body_part":"script","bindings":[{"type":"wasm_module","name":"wasmprogram","part":"wasmprogram"}]}"#
        );

        cleanup(out);
    }

    fn cleanup(path: PathBuf) {
        message::info(&format!("p: {:?}", path));
        fs::remove_dir_all(&path).unwrap();
        env::set_var("WORKER_BUNDLE_OUT", "".to_string());
    }
}
