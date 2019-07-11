use crate::workers::{Resource, Worker};
use reqwest::multipart;
use serde::Serialize;

#[derive(Debug)]
pub struct WorkerBundle {
    name: String,
    script_name: String,
    script_path: String,
    bindings: Vec<Binding>,
}

impl WorkerBundle {
    pub fn multipart(&self) -> Result<multipart::Form, failure::Error> {
        let mut form = multipart::Form::new();
        let parts = self.binding_parts()?;
        for (name, part) in parts {
            form = form.part(name, part);
        }
        form = form.file(self.script_name.clone(), &self.script_path)?;
        Ok(form)
    }

    fn binding_parts(&self) -> Result<Vec<(String, multipart::Part)>, failure::Error> {
        let mut parts: Vec<(String, multipart::Part)> = Vec::new();
        let meta_part = self.binding_meta_part()?;
        parts.push(("metadata".to_string(), meta_part));
        for binding in &self.bindings {
            match binding {
                Binding::wasm_module(ref wasm) => {
                    let part = multipart::Part::file(wasm.path.to_string())?;
                    parts.push((wasm.name.clone(), part));
                }
                Binding::kv_namespace(_) => {
                    // kv bindings don't add their own part to the multipart form
                }
            }
        }
        Ok(parts)
    }

    fn binding_meta_part(&self) -> Result<multipart::Part, failure::Error> {
        let metadata = MetaData {
            body_part: self.script_name.clone(),
            bindings: &self.bindings,
        };
        Ok(multipart::Part::bytes(serde_json::to_vec(&metadata)?)
            .file_name("metadata.json")
            .mime_str("application/json")?)
    }
}

impl From<Worker> for WorkerBundle {
    fn from(worker: Worker) -> WorkerBundle {
        WorkerBundle {
            name: worker.name,
            script_name: worker.script.name,
            script_path: worker.script.path,
            bindings: worker
                .resources
                .into_iter()
                .map(|r| Binding::from(r))
                .collect(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct MetaData<'a> {
    body_part: String,
    bindings: &'a [Binding],
}

// setting the serde tag to type makes it so the "type" key in the resulting json object is the
// same as the enum member name.
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Binding {
    #[allow(non_camel_case_types)]
    wasm_module(WasmBinding),
    #[allow(non_camel_case_types)]
    kv_namespace(KVBinding),
}

impl From<Resource> for Binding {
    fn from(resource: Resource) -> Binding {
        match resource {
            Resource::WasmModule(wasm) => Binding::wasm_module(WasmBinding {
                path: wasm.path.clone(),
                name: wasm.binding.clone(),
                part: wasm.binding.clone(),
            }),
            Resource::KVNamespace(kv) => Binding::kv_namespace(KVBinding {
                namespace_id: kv.namespace_id.clone(),
                name: kv.binding.clone(),
            }),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct WasmBinding {
    #[serde(skip_serializing)]
    path: String,
    name: String,
    part: String,
}

#[derive(Serialize, Debug)]
pub struct KVBinding {
    namespace_id: String,
    name: String,
}
