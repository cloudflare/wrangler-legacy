pub mod bundle;
pub mod output;

use crate::commands::publish::package::Package;
use crate::install;
use binary_install::Cache;
pub use bundle::Bundle;
use fs2::FileExt;
use log::info;
use output::WranglerjsOutput;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;
use std::fs;
use std::fs::File;
use std::iter;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::settings::project::Project;

use crate::terminal::message;

// Run the underlying {wranglerjs} executable.
//
// In Rust we create a virtual file, pass the pass to {wranglerjs}, run the
// executable and wait for completion. The file will receive the a serialized
// {WranglerjsOutput} struct.
// Note that the ability to pass a fd is platform-specific
pub fn run_build(cache: &Cache, project: &Project) -> Result<(), failure::Error> {
    let (mut command, temp_file, bundle) = setup_build(cache, project)?;

    info!("Running {:?}", command);

    let status = command.status()?;

    if status.success() {
        let output = fs::read_to_string(temp_file.clone()).expect("could not retrieve ouput");
        fs::remove_file(temp_file)?;

        let wranglerjs_output: WranglerjsOutput =
            serde_json::from_str(&output).expect("could not parse wranglerjs output");

        if wranglerjs_output.has_errors() {
            message::user_error(&format!("{}", wranglerjs_output.get_errors()));
            failure::bail!("Webpack returned an error");
        }

        bundle
            .write(&wranglerjs_output)
            .expect("could not write bundle to disk");

        let mut msg = format!(
            "Built successfully, script size is {}",
            wranglerjs_output.script_size()
        );
        if bundle.has_wasm() {
            msg = format!("{} and Wasm size is {}", msg, wranglerjs_output.wasm_size());
        }
        message::success(&msg);
        Ok(())
    } else {
        fs::remove_file(temp_file)?;
        failure::bail!("failed to execute `{:?}`: exited with {}", command, status)
    }
}

//setup a build to run wranglerjs, return the command, the ipc temp file, and the bundle
fn setup_build(
    cache: &Cache,
    project: &Project,
) -> Result<(Command, PathBuf, Bundle), failure::Error> {
    for tool in &["node", "npm"] {
        env_dep_installed(tool)?;
    }

    let current_dir = env::current_dir()?;
    run_npm_install(current_dir).expect("could not run `npm install`");

    let node = which::which("node").unwrap();
    let mut command = Command::new(node);
    let wranglerjs_path = install(cache).expect("could not install wranglerjs");
    command.arg(wranglerjs_path);

    //put path to our wasm_pack as env variable so wasm-pack-plugin can utilize it
    let wasm_pack_path = install::install("wasm-pack", "rustwasm", cache)?.binary("wasm-pack")?;
    command.env("WASM_PACK_PATH", wasm_pack_path);

    // create a temp file for IPC with the wranglerjs process
    let mut temp_file = env::temp_dir();
    temp_file.push(format!(".wranglerjs_output{}", random_chars(5)));
    File::create(temp_file.clone())?;

    command.arg(format!(
        "--output-file={}",
        temp_file.clone().to_str().unwrap().to_string()
    ));

    let bundle = Bundle::new();

    command.arg(format!("--wasm-binding={}", bundle.get_wasm_binding()));

    let webpack_config_path = PathBuf::from(
        &project
            .webpack_config
            .clone()
            .unwrap_or_else(|| "webpack.config.js".to_string()),
    );

    // if {webpack.config.js} is not present, we infer the entry based on the
    // {package.json} file and pass it to {wranglerjs}.
    // https://github.com/cloudflare/wrangler/issues/98
    if !bundle.has_webpack_config(&webpack_config_path) {
        let package = Package::new("./")?;
        let current_dir = env::current_dir()?;
        let package_main = current_dir
            .join(package.main()?)
            .to_str()
            .unwrap()
            .to_string();
        command.arg("--no-webpack-config=1");
        command.arg(format!("--use-entry={}", package_main));
    } else {
        command.arg(format!(
            "--webpack-config={}",
            &webpack_config_path.to_str().unwrap().to_string()
        ));
    }

    Ok((command, temp_file, bundle))
}

// Run {npm install} in the specified directory. Skips the install if a
// {node_modules} is found in the directory.
fn run_npm_install(dir: PathBuf) -> Result<(), failure::Error> {
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
fn env_dep_installed(tool: &str) -> Result<(), failure::Error> {
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
fn install(cache: &Cache) -> Result<PathBuf, failure::Error> {
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

fn random_chars(n: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(n)
        .collect()
}
