use reqwest::multipart;
use serde::Serialize;

pub struct Worker {
    script_path: String,
    bindings: Vec<Binding>,
}

impl Worker {
    pub fn multipart(&self) -> Result<multipart::Form, failure::Error> {
        let mut form = multipart::Form::new().file("script", &self.script_path)?;
        let binding_parts = binding_parts(&self.bindings)?;
        for (name, part) in binding_parts {
            form = form.part(name, part);
        }
        Ok(form)
    }
}

fn binding_parts<'a>(
    bindings: &'a Vec<Binding>,
) -> Result<Vec<(String, multipart::Part)>, failure::Error> {
    let mut parts: Vec<(String, multipart::Part)> = Vec::new();
    let meta_part = binding_meta_part(bindings)?;
    parts.push(("metadata".to_string(), meta_part));
    for binding in bindings {
        match binding {
            Binding::wasm_module(ref wasm) => {
                let mut part = multipart::Part::file(wasm.path.to_string())?;
                part = part.file_name(wasm.symbol.to_string());
                parts.push((wasm.symbol.to_string(), part));
            }
            Binding::kv_namespace(_) => {
                // kv bindings don't add their own part to the multipart form
            }
        }
    }
    Ok(parts)
}

fn binding_meta_part(bindings: &[Binding]) -> Result<multipart::Part, failure::Error> {
    let metadata = MetaData {
        body_part: "script".to_string(),
        bindings: bindings,
    };
    Ok(multipart::Part::bytes(serde_json::to_vec(&metadata)?).mime_str("application/json")?)
}

#[derive(Serialize, Debug)]
pub struct MetaData<'a> {
    body_part: String,
    bindings: &'a [Binding],
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum Binding {
    #[allow(non_camel_case_types)]
    wasm_module(WasmBinding),
    #[allow(non_camel_case_types)]
    kv_namespace(KVBinding),
}

#[derive(Serialize, Debug)]
pub struct WasmBinding {
    #[serde(skip_serializing)]
    path: String,
    #[serde(rename = "name")]
    symbol: String,
    part: String,
}

#[derive(Serialize, Debug)]
pub struct KVBinding {
    namespace_id: String,
    #[serde(rename = "name")]
    symbol: String,
}
