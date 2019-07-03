use flate2::write::ZlibEncoder;
use flate2::Compression;
use number_prefix::{NumberPrefix, Prefixed, Standalone};
use reqwest::multipart;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use crate::settings::binding::Binding;
use crate::settings::metadata;
use crate::terminal::message;

#[derive(Debug)]
pub struct WorkerBundle {
    // pass out directory, usesul for testing
    pub out_root: Option<PathBuf>,

    pub script_path: PathBuf,

    // FIXME: remove once rust backend can get the metadata from wrangler's
    // config.
    pub metadata_path: Option<PathBuf>,

    // TODO: document: dependecies...
    pub bindings: Vec<Binding>,
}

impl WorkerBundle {
    fn reduce_bindings<T>(&self, f: &Fn(&Binding, T) -> T, initial: T) -> T {
        let mut acc = initial;
        for binding in &self.bindings {
            acc = f(binding, acc);
        }
        acc
    }

    fn worker_out_path(&self) -> PathBuf {
        if let Some(value) = self.out_root.clone() {
            value
        } else {
            Path::new("./worker").to_path_buf()
        }
    }

    fn worker_metadata_path(&self) -> PathBuf {
        if let Some(value) = self.metadata_path.clone() {
            value
        } else {
            self.worker_out_path().join("metadata.json".to_string())
        }
    }

    fn write_metadata(&self) -> Result<(), failure::Error> {
        if self.metadata_path.is_none() {
            let json = serde_json::to_string(&metadata::Metadata {
                body_part: "script".to_string(),
                bindings: &self.bindings,
            });

            let metadata = json.expect("could not create metadata");
            let mut metadata_file = File::create(&self.worker_metadata_path()).expect(&format!(
                "could not create metadata file {:?}",
                &self.worker_metadata_path()
            ));
            metadata_file.write_all(metadata.as_bytes())?;
        }

        Ok(())
    }

    fn write_script(&self) -> Result<(), failure::Error> {
        let path = self.worker_out_path().join("script.js");
        let content = fs::read_to_string(&self.script_path)
            .expect(&format!("could not read file {:?}", &self.script_path));

        let mut file = File::create(&path).expect("could not create file");
        file.write_all(content.as_bytes())
            .expect("coult not write file");

        Ok(())
    }

    pub fn has_wasm(&self) -> bool {
        let is_wasm = |binding: &Binding, acc: bool| {
            if acc != true {
                match binding {
                    Binding::wasm_module(_) => true,
                }
            } else {
                acc
            }
        };
        self.reduce_bindings(&is_wasm, false)
    }

    pub fn persist(&self) -> Result<(), failure::Error> {
        self.persist_in()
    }

    pub fn persist_in(&self) -> Result<(), failure::Error> {
        println!("persist in {:?}", self.worker_out_path());
        if !self.worker_out_path().exists() {
            fs::create_dir_all(&self.worker_out_path()).expect(&format!(
                "could not create out directory {:?}",
                &self.worker_out_path()
            ));
        }

        self.write_script().expect("could not write script");
        self.write_metadata().expect("could not write metadata");

        let persist_binding = |binding: &Binding, _| match binding {
            Binding::wasm_module(ref wasm) => {
                // FIXME: doesn't support multi modules
                let path = self.worker_out_path().join("module.wasm".to_string());

                let bytes =
                    fs::read(&wasm.path).expect(&format!("could not read file {:?}", &wasm.path));
                let mut wasm_file = File::create(&path).expect("could not create file");
                wasm_file.write_all(&bytes).expect("coult not write file");
            }
        };

        Ok(self.reduce_bindings(&persist_binding, ()))
    }

    pub fn multipart(&self) -> Result<multipart::Form, failure::Error> {
        let form = multipart::Form::new()
            .file("metadata", self.worker_metadata_path())
            .unwrap_or_else(|_| {
                panic!(
                    "{:?} not found. Did you delete it?",
                    self.worker_metadata_path()
                )
            })
            .file("script", &self.script_path)
            .unwrap_or_else(|_| {
                panic!(
                    "{:?} not found. Did you rename your js files?",
                    self.script_path
                )
            });

        let binding_to_part = |binding: &Binding, form: multipart::Form| -> multipart::Form {
            match binding {
                Binding::wasm_module(wasm) => form
                    .file(wasm.name.clone(), wasm.path.clone())
                    .unwrap_or_else(|_| {
                        panic!("{:?} not found. Have you run wrangler build?", wasm.path)
                    }),
            }
        };
        Ok(self.reduce_bindings(&binding_to_part, form))
    }

    pub fn print_stats(&self) {
        let mut msg = format!("Built successfully, script size is {}", self.script_size());
        if self.has_wasm() {
            msg = format!("{} and Wasm size is {}", msg, self.wasm_size());
        }
        message::success(&msg);
    }

    pub fn script_size(&self) -> String {
        let script = fs::read(&self.script_path)
            .expect(&format!("could not read file {:?}", &self.script_path));
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(&script).expect("could not write buffer");
        let compressed_bytes = e.finish();

        match NumberPrefix::decimal(compressed_bytes.unwrap().len() as f64) {
            Standalone(bytes) => format!("{} bytes", bytes),
            Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
        }
    }

    pub fn wasm_size(&self) -> String {
        let sum = |binding: &Binding, size: usize| match binding {
            Binding::wasm_module(ref wasm) => {
                let wasm =
                    fs::read(&wasm.path).expect(&format!("could not read file {:?}", &wasm.path));
                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                e.write_all(&wasm).expect("could not write buffer");
                let compressed_bytes = e.finish();
                size + compressed_bytes.unwrap().len()
            } // , _ => size,
        };

        let total_size = self.reduce_bindings(&sum, 0);

        match NumberPrefix::decimal(total_size as f64) {
            Standalone(bytes) => format!("{} bytes", bytes),
            Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_returns_gzip_script_size() {
    //     let wranglerjs_output = WranglerjsOutput {
    //         errors: vec![],
    //         script: "aaaa".to_string(),
    //         dist_to_clean: None,
    //         wasm: None,
    //     };

    //     let bundle = wranglerjs_output
    //         .to_worker_bundle()
    //         .expect("could not create WorkerBundle");
    //     assert_eq!(bundle.script_size(), "12 bytes");
    // }

    // #[test]
    // fn it_returns_wasm_size() {
    //     let wranglerjs_output = WranglerjsOutput {
    //         errors: vec![],
    //         script: "".to_string(),
    //         dist_to_clean: None,
    //         wasm: Some("abc".to_string()),
    //     };

    //     let bundle = wranglerjs_output
    //         .to_worker_bundle()
    //         .expect("could not create WorkerBundle");
    //     assert_eq!(bundle.wasm_size(), "10 bytes");
    // }

}
