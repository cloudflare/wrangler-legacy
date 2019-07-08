pub struct Worker {
    pub name: String,
    pub script: Script,
    pub resources: Vec<Resource>,
}

pub struct Script {
    pub name: String,
    pub path: String,
}

pub enum Resource {
    WasmModule(WasmModule),
    KVNamespace(KVNamespace),
}

pub struct WasmModule {
    pub path: String,
    pub binding: String,
}

pub struct KVNamespace {
    pub namespace_id: String,
    pub binding: String,
}
