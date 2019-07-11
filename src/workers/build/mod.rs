mod krate;

use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{Resource, Script, WasmModule, Worker};

use crate::commands::build::wranglerjs;
use crate::commands::publish::package::Package;
use crate::settings::project::{Project, ProjectType};
use crate::terminal::message;
use crate::{commands, install};

pub fn build(project: &Project) -> Result<Worker, failure::Error> {
    match project.project_type {
        ProjectType::JavaScript => {
            message::info("JavaScript project found. Skipping unnecessary build!")
        }
        ProjectType::Rust => {
            let tool_name = "wasm-pack";
            let binary_path = install::install(tool_name, "rustwasm")?.binary(tool_name)?;
            let args = ["build", "--target", "no-modules"];

            let command = command(&args, binary_path);
            let command_name = format!("{:?}", command);

            commands::run(command, &command_name)?;
        }
        ProjectType::Webpack => {
            wranglerjs::run_build(project)?;
        }
    }

    let worker = match project.project_type {
        ProjectType::Rust => wasm_worker(&project.name)?,
        ProjectType::Webpack => webpack_worker(&project.name)?,
        ProjectType::JavaScript => js_worker(&project.name)?,
    };
    // add other resoureces
    Ok(worker)
}

fn wasm_worker(project_name: &str) -> Result<Worker, failure::Error> {
    let name = krate::Krate::new("./")?.name.replace("-", "_");

    build_generated_dir()?;
    concat_js(&name)?;

    let script_path = "./worker/generated/script.js";
    let wasm_path = format!("./pkg/{}_bg.wasm", name);

    let wasm_resource = wasm_resource(wasm_path)?;

    Ok(Worker {
        name: project_name.to_string(),
        script: Script {
            name: "script".to_string(),
            path: script_path.to_string(),
        },
        resources: vec![wasm_resource],
    })
}

fn webpack_worker(project_name: &str) -> Result<Worker, failure::Error> {
    let script_path = "./worker/script.js";
    let mut worker = Worker {
        name: project_name.to_string(),
        script: Script {
            name: "script".to_string(),
            path: script_path.to_string(),
        },
        resources: Vec::new(),
    };
    let wasm_path = "./worker/module.wasm";
    if Path::new(wasm_path).exists() {
        let wasm_resource = wasm_resource(wasm_path.to_string())?;
        worker.resources.push(wasm_resource);
    };
    Ok(worker)
}

fn js_worker(project_name: &str) -> Result<Worker, failure::Error> {
    let pkg = Package::new("./")?;
    let script_path = pkg.main()?;
    Ok(Worker {
        name: project_name.to_string(),
        script: Script {
            name: "script".to_string(),
            path: script_path,
        },
        resources: Vec::new(),
    })
}

fn wasm_resource(wasm_path: String) -> Result<Resource, failure::Error> {
    let metadata_path = "./worker/metadata_wasm.json";
    let mut binding = "wasm".to_string();

    if Path::new(metadata_path).exists() {
        let file = fs::File::open(metadata_path)?;
        let reader = BufReader::new(file);
        let data: serde_json::Value = serde_json::from_reader(reader)?;
        binding = match data["bindings"][0]["name"].as_str() {
            Some(s) => s.to_string(),
            None => {
                failure::bail!("binding was not a string");
            }
        };
    }
    Ok(Resource::WasmModule(WasmModule {
        path: wasm_path.to_string(),
        binding: binding,
    }))
}

fn concat_js(name: &str) -> Result<(), failure::Error> {
    let bindgen_js_path = format!("./pkg/{}.js", name);
    let bindgen_js: String = fs::read_to_string(bindgen_js_path)?.parse()?;

    let worker_js: String = fs::read_to_string("./worker/worker.js")?.parse()?;
    let js = format!("{} {}", bindgen_js, worker_js);

    fs::write("./worker/generated/script.js", js.as_bytes())?;
    Ok(())
}

fn build_generated_dir() -> Result<(), failure::Error> {
    let dir = "./worker/generated";
    if !Path::new(dir).is_dir() {
        fs::create_dir("./worker/generated")?;
    }
    Ok(())
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
