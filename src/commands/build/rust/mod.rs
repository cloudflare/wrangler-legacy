use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// FIXME: move krate in rust
use crate::commands::publish::krate;
use crate::settings::binding::Binding;
use crate::terminal::message;
use crate::worker_bundle::WorkerBundle;
use crate::{commands, install};

pub const WASM_BINDING: &str = "wasmprogram";

pub fn run_build() -> Result<WorkerBundle, failure::Error> {
    let tool_name = "wasm-pack";
    let binary_path = install::install(tool_name, "rustwasm")?.binary(tool_name)?;
    let args = ["build", "--target", "no-modules"];

    let command = command(&args, binary_path);
    let command_name = format!("{:?}", command);

    commands::run(command, &command_name)?;

    let name = krate::Krate::new("./")?.name.replace("-", "_");

    let mut bindings = vec![];
    bindings.push(Binding::new_wasm_module(
        Path::new(&format!("./pkg/{}_bg.wasm", name)).to_path_buf(), // path
        WASM_BINDING.to_string(),                                    // name
        WASM_BINDING.to_string(),                                    // part
    ));

    build_generated_dir()?;
    concat_js(&name)?;

    Ok(WorkerBundle {
        script_path: Path::new("./worker/generated/script.js").to_path_buf(),
        bindings,
        metadata_path: Some(Path::new("./worker/metadata_wasm.json").to_path_buf()),
        out_root: None,
    })
}

fn command(args: &[&str], binary_path: PathBuf) -> Command {
    message::working("Compiling your project to WebAssembly...");

    let mut c = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c.arg(binary_path);
        c
    } else {
        Command::new(binary_path)
    };

    c.args(args);
    c
}

fn build_generated_dir() -> Result<(), failure::Error> {
    let dir = "./worker/generated";
    if !Path::new(dir).is_dir() {
        fs::create_dir("./worker/generated")?;
    }
    Ok(())
}

fn concat_js(name: &str) -> Result<(), failure::Error> {
    let bindgen_js_path = format!("./pkg/{}.js", name);
    let bindgen_js: String = fs::read_to_string(bindgen_js_path)?.parse()?;

    // FIXME: this can't be the worker directory, moving it to the project
    // dir (./worker.js)?
    let worker_js: String = fs::read_to_string("./worker/worker.js")?.parse()?;
    let js = format!("{} {}", bindgen_js, worker_js);

    fs::write("./worker/generated/script.js", js.as_bytes())?;
    Ok(())
}
