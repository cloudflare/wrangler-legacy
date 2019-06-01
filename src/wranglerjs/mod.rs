use crate::commands::publish::package::Package;
use crate::install;
use binary_install::Cache;
use fs2::FileExt;
use log::info;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::iter;
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
    // Errors emited by {wranglerjs}, if any
    errors: Vec<String>,
}

impl WranglerjsOutput {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> String {
        self.errors.join("\n")
    }
}

// Directory where we should write the {Bundle}. It represents the built
// artifact.
const BUNDLE_OUT: &str = "./worker";
pub struct Bundle {
    out: String,
}

// We call a {Bundle} the output of a {Bundler}; representing what {Webpack}
// produces.
impl Bundle {
    pub fn new() -> Bundle {
        Bundle {
            out: BUNDLE_OUT.to_string(),
        }
    }

    #[cfg(test)]
    fn new_at(out: String) -> Bundle {
        Bundle { out }
    }

    pub fn write(&self, wranglerjs_output: WranglerjsOutput) -> Result<(), failure::Error> {
        let bundle_path = Path::new(&self.out);
        if !bundle_path.exists() {
            fs::create_dir(bundle_path)?;
        }

        let mut script_file = File::create(self.script_path())?;
        let mut script = create_prologue();
        script += &wranglerjs_output.script;

        if let Some(wasm) = wranglerjs_output.wasm {
            let mut wasm_file = File::create(self.wasm_path())?;
            wasm_file.write_all(wasm.as_bytes())?;
        }

        script_file.write_all(script.as_bytes())?;

        let mut metadata_file = File::create(self.metadata_path())?;
        metadata_file.write_all(create_metadata(self).as_bytes())?;

        // cleanup {Webpack} dist, if specified.
        if let Some(dist_to_clean) = wranglerjs_output.dist_to_clean {
            info!("Remove {}", dist_to_clean);
            fs::remove_dir_all(dist_to_clean).expect("could not clean Webpack dist.");
        }

        Ok(())
    }

    pub fn metadata_path(&self) -> String {
        Path::new(&self.out)
            .join("metadata.json".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn wasm_path(&self) -> String {
        Path::new(&self.out)
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
        Path::new(&self.out)
            .join("script.js".to_string())
            .to_str()
            .unwrap()
            .to_string()
    }
}

fn random_chars(n: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(n)
        .collect()
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
    let node = which::which("node").unwrap();
    let mut command = Command::new(node);
    command.arg(wranglerjs_path);
    command.env("WASM_PACK_PATH", wasm_pack_path);

    // create temp file for special {wranglerjs} IPC.
    let mut temp_file = env::temp_dir();
    temp_file.push(format!(".wranglerjs_output{}", random_chars(5)));
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
    let flock_path = dir.join(&".install.lock");
    let flock = File::create(&flock_path)?;
    // avoid running multiple {npm install} at the same time (eg. in tests)
    flock.lock_exclusive()?;

    if !dir.join("node_modules").exists() {
        let mut command = build_npm_command();
        command.current_dir(dir.clone());
        command.arg("install");
        info!("Running {:?} in directory {:?}", command, dir);

        let status = command.status()?;

        if !status.success() {
            failure::bail!("failed to execute `{:?}`: exited with {}", command, status)
        }
    } else {
        info!("skipping npm install because node_modules exists");
    }

    // TODO(sven): figure out why the file doesn't exits in some cases? Even if
    // the thread should have locked it.
    if flock_path.exists() {
        fs::remove_file(&flock_path)?;
    }
    flock.unlock()?;

    Ok(())
}

// build a Command for npm
//
// Here's the deal: on Windows, `npm` isn't a binary, it's a shell script.
// This means that we can't invoke it via `Command` directly on Windows,
// we need to invoke `cmd /C npm`, to run it within the cmd environment.
fn build_npm_command() -> Command {
    if install::target::WINDOWS {
        let mut command = Command::new("cmd");
        command.arg("/C");
        command.arg("npm");

        command
    } else {
        Command::new("npm")
    }
}

// Ensures the specified tool is available in our env.
pub fn env_dep_installed(tool: &str) -> Result<(), failure::Error> {
    if which::which(tool).is_err() {
        failure::bail!("You need to install {}", tool)
    }
    Ok(())
}

// Use the env-provided source directory and remove the quotes
fn get_source_dir() -> PathBuf {
    let mut dir = install::target::SOURCE_DIR.to_string();
    dir.remove(0);
    dir.remove(dir.len() - 1);
    Path::new(&dir).to_path_buf()
}

// Install {wranglerjs} from our GitHub releases
pub fn install(cache: &Cache) -> Result<PathBuf, failure::Error> {
    let wranglerjs_path = if install::target::DEBUG {
        let source_path = get_source_dir();
        let wranglerjs_path = source_path.join("wranglerjs");
        info!("wranglerjs at: {:?}", wranglerjs_path);
        wranglerjs_path
    } else {
        let tool_name = "wranglerjs";
        let version = env!("CARGO_PKG_VERSION");
        let wranglerjs_path = install::install_artifact(tool_name, "cloudflare", cache, version)?;
        info!("wranglerjs downloaded at: {:?}", wranglerjs_path.path());
        wranglerjs_path.path()
    };

    run_npm_install(wranglerjs_path.clone()).expect("could not install wranglerjs dependencies");
    Ok(wranglerjs_path)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temp_dir(name: &str) -> String {
        let mut dir = env::temp_dir();
        dir.push(name);
        fs::create_dir(&dir).expect("could not create temp dir");

        dir.to_str().unwrap().to_string()
    }

    #[test]
    fn it_writes_the_bundle_metadata() {
        let out = create_temp_dir("it_writes_the_bundle_metadata");
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "".to_string(),
            dist_to_clean: None,
            wasm: None,
        };
        let bundle = Bundle::new_at(out.clone());

        bundle.write(wranglerjs_output).unwrap();
        assert!(Path::new(&bundle.metadata_path()).exists());

        cleanup(out);
    }

    #[test]
    fn it_writes_the_bundle_script() {
        let out = create_temp_dir("it_writes_the_bundle_script");
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "foo".to_string(),
            dist_to_clean: None,
            wasm: None,
        };
        let bundle = Bundle::new_at(out.clone());

        bundle.write(wranglerjs_output).unwrap();
        assert!(Path::new(&bundle.script_path()).exists());
        assert!(!Path::new(&bundle.wasm_path()).exists());

        cleanup(out);
    }

    #[test]
    fn it_writes_the_bundle_wasm() {
        let out = create_temp_dir("it_writes_the_bundle_wasm");
        let wranglerjs_output = WranglerjsOutput {
            errors: vec![],
            script: "".to_string(),
            wasm: Some("abc".to_string()),
            dist_to_clean: None,
        };
        let bundle = Bundle::new_at(out.clone());

        bundle.write(wranglerjs_output).unwrap();
        assert!(Path::new(&bundle.wasm_path()).exists());
        assert!(bundle.has_wasm());

        cleanup(out);
    }

    #[test]
    fn it_has_errors() {
        let wranglerjs_output = WranglerjsOutput {
            errors: vec!["a".to_string(), "b".to_string()],
            script: "".to_string(),
            wasm: None,
            dist_to_clean: None,
        };
        assert!(wranglerjs_output.has_errors());
        assert!(wranglerjs_output.get_errors() == "a\nb");
    }

    fn cleanup(name: String) {
        let current_dir = env::current_dir().unwrap();
        let path = Path::new(&current_dir).join(name);
        println!("p: {:?}", path);
        fs::remove_dir_all(path).unwrap();
    }
}
