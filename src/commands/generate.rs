use std::fs;
use std::io::Write;
use std::process::Command;

pub fn generate() -> Result<(), failure::Error> {
    let worker_init =
        "cargo generate --git https://github.com/rustwasm/wasm-pack-template.git --name wasm-worker";

    let _output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", worker_init]).output()?
    } else {
        Command::new("sh").arg("-c").arg(worker_init).output()?
    };
    fs::create_dir("./wasm-worker/worker")?;
    write_files()?;
    Ok(())
}

fn write_files() -> Result<(), failure::Error> {
    write_metadata()?;
    write_worker()?;
    Ok(())
}

fn write_metadata() -> Result<(), failure::Error> {
    let metadata = r#"{
    "body_part": "script",
    "bindings": [
        {
            "name": "wasm",
            "type": "wasm_module",
            "part": "wasmprogram"
        }
    ]
}"#;

    let mut metadata_file = fs::File::create("./wasm-worker/worker/metadata_wasm.json")?;
    metadata_file.write_all(metadata.as_bytes())?;
    Ok(())
}

fn write_worker() -> Result<(), failure::Error> {
    let worker = r#"addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
    const { greet } = wasm_bindgen;
    await wasm_bindgen(wasm)
    const greeting = greet()
    return new Response(greeting, {status: 200})
}"#;

    let mut metadata_file = fs::File::create("./wasm-worker/worker/worker.js")?;
    metadata_file.write_all(worker.as_bytes())?;
    Ok(())
}
