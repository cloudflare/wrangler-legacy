mod bundle;
pub mod output;

use crate::commands::build::watch::wait_for_changes;
use crate::commands::build::watch::COOLDOWN_PERIOD;
use crate::commands::generate::run_generate;

use crate::commands::publish::package::Package;
use crate::install;
use crate::util;
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

use crate::settings::target::Target;
use crate::terminal::message;

use notify::{self, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

// Run the underlying {wranglerjs} executable.
//
// In Rust we create a virtual file, pass the pass to {wranglerjs}, run the
// executable and wait for completion. The file will receive the a serialized
// {WranglerjsOutput} struct.
// Note that the ability to pass a fd is platform-specific
pub fn run_build(target: &Target) -> Result<(), failure::Error> {
    let (mut command, temp_file, bundle) = setup_build(target)?;

    info!("Running {:?}", command);

    let status = command.status()?;

    if status.success() {
        let output = fs::read_to_string(temp_file).expect("could not retrieve output");

        let wranglerjs_output: WranglerjsOutput =
            serde_json::from_str(&output).expect("could not parse wranglerjs output");

        write_wranglerjs_output(&bundle, &wranglerjs_output)
    } else {
        failure::bail!("failed to execute `{:?}`: exited with {}", command, status)
    }
}

pub fn run_build_and_watch(target: &Target, tx: Option<Sender<()>>) -> Result<(), failure::Error> {
    let (mut command, temp_file, bundle) = setup_build(target)?;
    command.arg("--watch=1");

    let is_site = target.site.clone();

    info!("Running {:?} in watch mode", command);

    //Turbofish the result of the closure so we can use ?
    thread::spawn::<_, Result<(), failure::Error>>(move || {
        let _command_guard = util::GuardedCommand::spawn(command);

        let (watcher_tx, watcher_rx) = channel();
        let mut watcher = notify::watcher(watcher_tx, Duration::from_secs(1))?;

        watcher.watch(&temp_file, RecursiveMode::Recursive)?;
        info!("watching temp file {:?}", &temp_file);

        if let Some(site) = is_site {
            let bucket = site.bucket;
            if Path::new(&bucket).exists() {
                watcher.watch(&bucket, RecursiveMode::Recursive)?;
                info!("watching static sites asset file {:?}", &bucket);
            } else {
                failure::bail!(
                    "Attempting to watch static assets bucket \"{}\" which doesn't exist",
                    bucket
                );
            }
        }

        let mut is_first = true;

        loop {
            match wait_for_changes(&watcher_rx, COOLDOWN_PERIOD) {
                Ok(_) => {
                    if is_first {
                        is_first = false;
                        message::info("Ignoring stale first change");
                        //skip the first change event
                        //so we don't do a refresh immediately
                        continue;
                    }

                    let output = fs::read_to_string(&temp_file).expect("could not retrieve ouput");
                    let wranglerjs_output: WranglerjsOutput =
                        serde_json::from_str(&output).expect("could not parse wranglerjs output");

                    if write_wranglerjs_output(&bundle, &wranglerjs_output).is_ok() {
                        if let Some(tx) = tx.clone() {
                            tx.send(()).expect("--watch change message failed to send");
                        }
                    }
                }
                Err(_) => message::user_error("Something went wrong while watching."),
            }
        }
    });

    Ok(())
}

fn write_wranglerjs_output(
    bundle: &Bundle,
    output: &WranglerjsOutput,
) -> Result<(), failure::Error> {
    if output.has_errors() {
        message::user_error(output.get_errors().as_str());
        failure::bail!("Webpack returned an error");
    }

    bundle.write(output)?;

    let msg = format!(
        "Built successfully, built project size is {}",
        output.project_size()
    );

    message::success(&msg);
    Ok(())
}

//setup a build to run wranglerjs, return the command, the ipc temp file, and the bundle
fn setup_build(target: &Target) -> Result<(Command, PathBuf, Bundle), failure::Error> {
    for tool in &["node", "npm"] {
        env_dep_installed(tool)?;
    }

    let build_dir = target.build_dir()?;

    if target.site.is_some() {
        scaffold_site_worker(&target)?;
    }

    run_npm_install(&build_dir).expect("could not run `npm install`");

    let node = which::which("node").unwrap();
    let mut command = Command::new(node);
    let wranglerjs_path = install().expect("could not install wranglerjs");
    command.arg(wranglerjs_path);

    //put path to our wasm_pack as env variable so wasm-pack-plugin can utilize it
    let wasm_pack_path = install::install("wasm-pack", "rustwasm")?.binary("wasm-pack")?;
    command.env("WASM_PACK_PATH", wasm_pack_path);

    // create a temp file for IPC with the wranglerjs process
    let mut temp_file = env::temp_dir();
    temp_file.push(format!(".wranglerjs_output{}", random_chars(5)));
    File::create(temp_file.clone())?;

    command.arg(format!(
        "--output-file={}",
        temp_file.clone().to_str().unwrap().to_string()
    ));

    let bundle = Bundle::new(&build_dir);

    command.arg(format!("--wasm-binding={}", bundle.get_wasm_binding()));

    let webpack_config_path = if let Some(webpack_config) = &target.webpack_config {
        // require webpack_config in wrangler.toml to use it in sites
        Some(PathBuf::from(&webpack_config))
    } else if target.site.is_none() {
        let config_path = PathBuf::from("webpack.config.js".to_string());
        // backwards compatibility, deprecated in 1.6.0
        // if webpack.config.js exists and is not specified in wrangler.toml, use it and warn
        if bundle.has_webpack_config(&config_path) {
            message::warn("In Wrangler v1.6.0, you will need to include a webpack_config field in your wrangler.toml to build with a custom webpack configuration.");
            Some(config_path)
        } else {
            // if webpack.config.js does not exist, don't warn, use our default
            None
        }
    } else {
        // don't use `webpack.config.js` if this project is a site
        None
    };

    // if {webpack.config.js} is not present, we infer the entry based on the
    // {package.json} file and pass it to {wranglerjs}.
    // https://github.com/cloudflare/wrangler/issues/98
    if let Some(webpack_config_path) = webpack_config_path {
        build_with_custom_webpack(&mut command, &webpack_config_path);
    } else {
        build_with_default_webpack(&mut command, &build_dir)?;
    }

    Ok((command, temp_file, bundle))
}

fn build_with_custom_webpack(command: &mut Command, webpack_config_path: &PathBuf) {
    command.arg(format!(
        "--webpack-config={}",
        &webpack_config_path.to_str().unwrap().to_string()
    ));
}

fn build_with_default_webpack(
    command: &mut Command,
    build_dir: &PathBuf,
) -> Result<(), failure::Error> {
    let package = Package::new(&build_dir)?;
    let package_main = build_dir
        .join(package.main(&build_dir)?)
        .to_str()
        .unwrap()
        .to_string();
    command.arg("--no-webpack-config=1");
    command.arg(format!("--use-entry={}", package_main));
    Ok(())
}

pub fn scaffold_site_worker(target: &Target) -> Result<(), failure::Error> {
    let build_dir = target.build_dir()?;
    // TODO: this is a placeholder template. Replace with The Real Thing on launch.
    let template = "https://github.com/cloudflare/worker-sites-init";

    if !Path::new(&build_dir).exists() {
        // TODO: use site.entry_point instead of build_dir explicitly.
        run_generate(build_dir.file_name().unwrap().to_str().unwrap(), template)?;

        // This step is to prevent having a git repo within a git repo after
        // generating the scaffold into an existing project.
        fs::remove_dir_all(&build_dir.join(".git"))?;
    }

    Ok(())
}

// Run {npm install} in the specified directory. Skips the install if a
// {node_modules} is found in the directory.
fn run_npm_install(dir: &PathBuf) -> Result<(), failure::Error> {
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

    // TODO(sven): figure out why the file doesn't exits in some cases?
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
fn install() -> Result<PathBuf, failure::Error> {
    let wranglerjs_path = if install::target::DEBUG {
        let source_path = get_source_dir();
        let wranglerjs_path = source_path.join("wranglerjs");
        info!("wranglerjs at: {:?}", wranglerjs_path);
        wranglerjs_path
    } else {
        let tool_name = "wranglerjs";
        let version = env!("CARGO_PKG_VERSION");
        let wranglerjs_path = install::install_artifact(tool_name, "cloudflare", version)?;
        info!("wranglerjs downloaded at: {:?}", wranglerjs_path.path());
        wranglerjs_path.path()
    };

    run_npm_install(&wranglerjs_path.clone()).expect("could not install wranglerjs dependencies");
    Ok(wranglerjs_path)
}

fn random_chars(n: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(n)
        .collect()
}
