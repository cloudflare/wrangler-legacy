use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use log::info;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

use crate::terminal::emoji;
use crate::commands::build::wranglerjs;
use crate::commands::publish::package::Package;
use crate::terminal::message;
use crate::worker::{Resource, Script, WasmModule, Worker};
use crate::{commands, install};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: ProjectType,
    pub zone_id: Option<String>,
    pub private: Option<bool>,
    pub webpack_config: Option<String>,
    pub account_id: String,
    pub route: Option<String>,
    pub routes: Option<HashMap<String, String>>,
    #[serde(rename = "kv-namespaces")]
    pub kv_namespaces: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    JavaScript,
    Rust,
    Webpack,
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::Webpack
    }
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            ProjectType::JavaScript => "js",
            ProjectType::Rust => "rust",
            ProjectType::Webpack => "webpack",
        };
        write!(f, "{}", printable)
    }
}

impl FromStr for ProjectType {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "javascript" => Ok(ProjectType::JavaScript),
            "rust" => Ok(ProjectType::Rust),
            "webpack" => Ok(ProjectType::Webpack),
            _ => failure::bail!("{} is not a valid wrangler project type!", s),
        }
    }
}

impl Project {
    pub fn generate(
        name: String,
        project_type: ProjectType,
        init: bool,
    ) -> Result<Project, failure::Error> {
        let project = Project {
            name: name.clone(),
            project_type: project_type.clone(),
            private: Some(false),
            zone_id: Some(String::new()),
            account_id: String::new(),
            route: Some(String::new()),
            routes: None,
            kv_namespaces: None,
            webpack_config: None,
        };

        let toml = toml::to_string(&project)?;
        let config_path = if init {
            PathBuf::from("./")
        } else {
            Path::new("./").join(&name)
        };
        let config_file = config_path.join("wrangler.toml");

        info!("Writing a wrangler.toml file at {}", config_file.display());
        fs::write(&config_file, &toml)?;
        Ok(project)
    }

    pub fn new() -> Result<Self, failure::Error> {
        get_project_config()
    }

    pub fn build(&self) -> Result<(), failure::Error> {
        match self.project_type {
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
                wranglerjs::run_build(self)?;
            }
        }

        Ok(())
    }

    pub fn worker(&self) -> Result<Worker, failure::Error> {
        self.build()?;
        let worker = match self.project_type {
            ProjectType::Rust => self.wasm_worker()?,
            ProjectType::Webpack => self.webpack_worker()?,
            ProjectType::JavaScript => self.js_worker()?,
        };
        // add other resoureces
        Ok(worker)
    }

    fn wasm_worker(&self) -> Result<Worker, failure::Error> {
        let name = krate::Krate::new("./")?.name.replace("-", "_");

        build_generated_dir()?;
        concat_js(&name)?;

        let script_path = "./worker/generated/script.js";
        let wasm_path = format!("./pkg/{}_bg.wasm", name);

        let wasm_resource = wasm_resource(wasm_path)?;

        Ok(Worker {
            name: self.name.clone(),
            script: Script {
                name: "script".to_string(),
                path: script_path.to_string(),
            },
            resources: vec![wasm_resource],
        })
    }

    fn webpack_worker(&self) -> Result<Worker, failure::Error> {
        let script_path = "./worker/script.js";
        let mut worker = Worker {
            name: self.name.clone(),
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

    fn js_worker(&self) -> Result<Worker, failure::Error> {
        let pkg = Package::new("./")?;
        let script_path = pkg.main()?;
        Ok(Worker {
            name: self.name.clone(),
            script: Script {
                name: "script".to_string(),
                path: script_path,
            },
            resources: Vec::new(),
        })
    }
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

pub fn get_project_config() -> Result<Project, failure::Error> {
    let mut s = Config::new();

    let config_path = Path::new("./wrangler.toml");
    let config_str = config_path
        .to_str()
        .expect("project config path should be a string");
    s.merge(File::with_name(config_str))?;

    // Eg.. `CF_ACCOUNT_AUTH_KEY=farts` would set the `account_auth_key` key
    s.merge(Environment::with_prefix("CF"))?;

    let project: Result<Project, config::ConfigError> = s.try_into();
    match project {
        Ok(s) => Ok(s),
        Err(e) => {
            let msg = format!(
                "{} Your project config has an error, check your `wrangler.toml`: {}",
                emoji::WARN,
                e
            );

            failure::bail!(msg)
        }
    }
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

    let worker_js: String = fs::read_to_string("./worker/worker.js")?.parse()?;
    let js = format!("{} {}", bindgen_js, worker_js);

    fs::write("./worker/generated/script.js", js.as_bytes())?;
    Ok(())
}
