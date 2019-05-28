use crate::commands::publish::package::Package;
use crate::install;
use binary_install::Cache;
use log::info;
use serde::Deserialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

// This structure represents the communication between {wranglerjs} and
// {wrangler}. It is send back after {wranglerjs} completion.
// FIXME(sven): make this private
#[derive(Deserialize, Debug)]
pub struct WranglerjsOutput {
    wasm: Option<String>,
    script: String,
    // {wranglerjs} will send us the path to the {dist} directory that {Webpack}
    // used; it's tedious to remove a directory with content in JavaScript so
    // let's do it in Rust!
    dist_to_clean: Option<String>,
}

impl WranglerjsOutput {}

// Directory where we should write the {Bundle}. It represents the built
// artifact.
const BUNDLE_OUT: &str = "./worker";
pub struct Bundle {}

// We call a {Bundle} the output of a {Bundler}; representing what {Webpack}
// produces.
impl Bundle {
    pub fn new() -> Bundle {
        Bundle {}
    }

    pub fn write(&self, wranglerjs_output: WranglerjsOutput) -> Result<(), failure::Error> {
        let bundle_path = Path::new(BUNDLE_OUT);
        if !bundle_path.exists() {
            fs::create_dir(bundle_path)?;
        }

        let mut metadata_file = File::create(self.metadata_path())?;
        metadata_file.write_all(create_metadata(self).as_bytes())?;

        let mut script_file = File::create(self.script_path())?;
        let mut script = create_prologue();
        script += &wranglerjs_output.script;

        if let Some(wasm) = wranglerjs_output.wasm {
            let mut wasm_file = File::create(self.wasm_path())?;
            wasm_file.write_all(wasm.as_bytes())?;
        }

        script_file.write_all(script.as_bytes())?;

        // cleanup {Webpack} dist, if specified.
        if let Some(dist_to_clean) = wranglerjs_output.dist_to_clean {
            info!("Remove {}", dist_to_clean);
            fs::remove_dir_all(dist_to_clean).expect("could not clean Webpack dist.");
        }

        Ok(())
    }

    pub fn metadata_path(&self) -> String {
        Path::new(BUNDLE_OUT)
            .join("metadata.json".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn wasm_path(&self) -> String {
        Path::new(BUNDLE_OUT)
            .join("module.wasm".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn has_wasm(&self) -> bool {
        Path::new(&self.wasm_path()).exists()
    }

    pub fn has_webpack_config(&self) -> bool {
        Path::new("webpack.config.js").exists()
    }

    pub fn get_wasm_binding(&self) -> String {
        "wasmprogram".to_string()
    }

    pub fn script_path(&self) -> String {
        Path::new(BUNDLE_OUT)
            .join("script.js".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }
}

// Run the underlying {wranglerjs} executable.
//
// In Rust we create a virtual file, pass the pass to {wranglerjs}, run the
// executable and wait for completion. The file will receive the a serialized
// {WranglerjsOutput} struct.
// Note that the ability to pass a fd is platform-specific
pub fn run_build(
    wranglerjs_path: PathBuf,
    wasm_pack_path: PathBuf,
    bundle: &Bundle,
) -> Result<WranglerjsOutput, failure::Error> {
    if !Path::new(BUNDLE_OUT).exists() {
        fs::create_dir(BUNDLE_OUT)?;
    }

    let node = which::which("node").unwrap();
    let mut command = Command::new(node);
    command.arg(wranglerjs_path);
    command.env("WASM_PACK_PATH", wasm_pack_path);

    // create temp file for special {wranglerjs} IPC.
    let mut temp_file = env::temp_dir();
    temp_file.push(".wranglerjs_output");
    File::create(temp_file.clone())?;

    command.arg(format!(
        "--output-file={}",
        temp_file.clone().to_str().unwrap().to_string()
    ));
    command.arg(format!("--wasm-binding={}", bundle.get_wasm_binding()));

    // if {webpack.config.js} is not present, we infer the entry based on the
    // {package.json} file and pass it to {wranglerjs}.
    // https://github.com/cloudflare/wrangler/issues/98
    if !bundle.has_webpack_config() {
        let package = Package::new("./")?;
        let current_dir = env::current_dir()?;
        let package_main = current_dir
            .join(package.main()?)
            .to_str()
            .unwrap()
            .to_string();
        command.arg("--no-webpack-config=1");
        command.arg(format!("--use-entry={}", package_main));
    }

    info!("Running {:?}", command);

    let status = command.status()?;
    let output = fs::read_to_string(temp_file.clone()).expect("could not retrieve ouput");
    fs::remove_file(temp_file)?;

    if status.success() {
        Ok(serde_json::from_str(&output).expect("could not parse wranglerjs output"))
    } else {
        failure::bail!("failed to execute `{:?}`: exited with {}", command, status)
    }
}

// Run {npm install} in the specified directory. Skips the install if a
// {node_modules} is found in the directory.
pub fn run_npm_install(dir: PathBuf) -> Result<(), failure::Error> {
    if dir.join("node_modules").exists() {
        info!("skipping npm install because node_modules exists");
        return Ok(());
    }

    let mut command = Command::new("npm");
    command.current_dir(dir);
    command.arg("install");
    info!("Running {:?}", command);

    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        failure::bail!("failed to execute `{:?}`: exited with {}", command, status)
    }
}

// Ensures the specified tool is available in our env.
pub fn env_dep_installed(tool: &str) -> Result<(), failure::Error> {
    if which::which(tool).is_err() {
        failure::bail!("You need to install {}", tool)
    }
    Ok(())
}

// Install {wranglerjs} from our GitHub releases
pub fn install(cache: &Cache) -> Result<PathBuf, failure::Error> {
    let tool_name = "wranglerjs";
    let wranglerjs_path = install::install_artifact(tool_name, "cloudflare", cache)?;
    info!("wranglerjs downloaded at: {:?}", wranglerjs_path.path());

    run_npm_install(wranglerjs_path.path()).expect("could not install wranglerjs dependecies");

    Ok(wranglerjs_path.path())
}

// We inject some code at the top-level of the Worker; called {prologue}.
// This aims to provide additional support, for instance providing {window}.
pub fn create_prologue() -> String {
    r#"
        const window = this;
    "#
    .to_string()
}

// This metadata describe the bindings on the Worker.
fn create_metadata(bundle: &Bundle) -> String {
    info!("create metadata; wasm={}", bundle.has_wasm());
    if bundle.has_wasm() {
        format!(
            r#"
                {{
                    "body_part": "script",
                    "binding": {{
                        "name": "{name}",
                        "type": "wasm_module",
                        "part": "{name}"
                    }}
                }}
            "#,
            name = bundle.get_wasm_binding(),
        )
        .to_string()
    } else {
        r#"
                {
                    "body_part": "script"
                }
            "#
        .to_string()
    }
}

// FIXME(sven): doesn't work because they have a race for the BUNDLE_OUT,
// make it configurable
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_writes_the_bundle_metadata() {
//         let wranglerjs_output = WranglerjsOutput {
//             script: "".to_string(),
//             dist_to_clean: None,
//             wasm: None,
//         };
//         let bundle = Bundle::new();

//         bundle.write(wranglerjs_output).unwrap();
//         assert!(Path::new(&bundle.metadata_path()).exists());

//         cleanup(BUNDLE_OUT);
//     }

//     #[test]
//     fn it_writes_the_bundle_script() {
//         let wranglerjs_output = WranglerjsOutput {
//             script: "foo".to_string(),
//             dist_to_clean: None,
//             wasm: None,
//         };
//         let bundle = Bundle::new();

//         bundle.write(wranglerjs_output).unwrap();
//         assert!(Path::new(&bundle.script_path()).exists());
//         assert!(!Path::new(&bundle.wasm_path()).exists());

//         cleanup(BUNDLE_OUT);
//     }

//     #[test]
//     fn it_writes_the_bundle_wasm() {
//         let wranglerjs_output = WranglerjsOutput {
//             script: "".to_string(),
//             wasm: Some("abc".to_string()),
//             dist_to_clean: None,
//         };
//         let bundle = Bundle::new();

//         bundle.write(wranglerjs_output).unwrap();
//         assert!(Path::new(&bundle.wasm_path()).exists());
//         assert!(bundle.has_wasm());

//         cleanup(BUNDLE_OUT);
//     }

//     fn cleanup(name: &str) {
//         let current_dir = env::current_dir().unwrap();
//         let path = Path::new(&current_dir).join(name);
//         println!("p: {:?}", path);
//         fs::remove_dir_all(path).unwrap();
//     }
// }
