#[derive(Debug)]
pub struct Worker {
    pub name: String,
    pub script: Script,
    pub resources: Vec<Resource>,
}

#[derive(Debug)]
pub struct Script {
    pub name: String,
    pub path: String,
}

#[derive(Debug)]
pub enum Resource {
    WasmModule(WasmModule),
    KVNamespace(KVNamespace),
}

#[derive(Debug)]
pub struct WasmModule {
    pub path: String,
    pub binding: String,
}

#[derive(Debug)]
pub struct KVNamespace {
    pub namespace_id: String,
    pub binding: String,
}
