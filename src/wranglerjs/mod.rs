mod bundle;
mod guarded_command;
pub mod output;

pub use bundle::Bundle;

use std::env;
use std::fs;
use std::fs::File;
use std::iter;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use fs2::FileExt;
use notify::{self, RecursiveMode, Watcher};
use output::WranglerjsOutput;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use semver::Version;

use crate::install;
use crate::settings::toml::Target;
use crate::terminal::message::{Message, StdErr, StdOut};
use crate::upload::package::Package;
use crate::watch::{wait_for_changes, COOLDOWN_PERIOD};

use guarded_command::GuardedCommand;

// Run the underlying {wranglerjs} executable.

// In Rust we create a virtual file, pass it to {wranglerjs}, run the
// executable and wait for completion. The file will receive a serialized
// {WranglerjsOutput} struct.
// Note that the ability to pass a fd is platform-specific
pub fn run_build(target: &Target) -> Result<WranglerjsOutput> {
    let (mut command, temp_file, bundle) = setup_build(target)?;

    log::info!("Running {:?}", command);

    let status = command.status()?;

    if status.success() {
        let output = fs::read_to_string(&temp_file).expect("could not retrieve output");
        fs::remove_file(temp_file)?;
        let wranglerjs_output: WranglerjsOutput =
            serde_json::from_str(&output).expect("could not parse wranglerjs output");

        let custom_webpack = target.webpack_config.is_some();
        write_wranglerjs_output(&bundle, &wranglerjs_output, custom_webpack)?;
        Ok(wranglerjs_output)
    } else {
        anyhow::bail!("failed to execute `{:?}`: exited with {}", command, status)
    }
}

pub fn run_build_and_watch(target: &Target, tx: Option<Sender<()>>) -> Result<()> {
    let (mut command, temp_file, bundle) = setup_build(target)?;
    command.arg("--watch=1");

    let is_site = target.site.clone();
    let custom_webpack = target.webpack_config.is_some();

    log::info!("Running {:?} in watch mode", command);

    // Turbofish the result of the closure so we can use ?
    thread::spawn::<_, Result<()>>(move || {
        let _command_guard = GuardedCommand::spawn(command);

        let (watcher_tx, watcher_rx) = channel();
        let mut watcher = notify::watcher(watcher_tx, Duration::from_secs(1))?;

        watcher.watch(&temp_file, RecursiveMode::Recursive)?;
        log::info!("watching temp file {:?}", &temp_file);

        if let Some(site) = is_site {
            let bucket = site.bucket;
            if Path::new(&bucket).exists() {
                watcher.watch(&bucket, RecursiveMode::Recursive)?;
                log::info!("watching static sites asset file {:?}", &bucket);
            } else {
                anyhow::bail!(
                    "Attempting to watch static assets bucket \"{}\" which doesn't exist",
                    bucket.display()
                );
            }
        }

        let mut is_first = true;

        loop {
            match wait_for_changes(&watcher_rx, None, COOLDOWN_PERIOD) {
                Ok(_) => {
                    if is_first {
                        is_first = false;
                        StdOut::info("Ignoring stale first change");
                        // skip the first change event
                        // so we don't do a refresh immediately
                        continue;
                    }

                    let output = fs::read_to_string(&temp_file).expect("could not retrieve output");
                    let wranglerjs_output: WranglerjsOutput =
                        serde_json::from_str(&output).expect("could not parse wranglerjs output");

                    if write_wranglerjs_output(&bundle, &wranglerjs_output, custom_webpack).is_ok()
                    {
                        if let Some(tx) = tx.clone() {
                            if let Err(e) = tx.send(()) {
                                log::error!("wranglerjs watch operation failed to notify: {}", e);
                            }
                        }
                    }
                }
                Err(_) => StdOut::user_error("Something went wrong while watching."),
            }
        }
    });

    Ok(())
}

fn write_wranglerjs_output(
    bundle: &Bundle,
    output: &WranglerjsOutput,
    custom_webpack: bool,
) -> Result<()> {
    if output.has_errors() {
        StdErr::user_error(output.get_errors().as_str());
        if custom_webpack {
            anyhow::bail!(
            "webpack returned an error. Try configuring `entry` in your webpack config relative to the current working directory, or setting `context = __dirname` in your webpack config."
        );
        } else {
            anyhow::bail!(
            "webpack returned an error. You may be able to resolve this issue by running npm install."
        );
        }
    }

    bundle.write(output)?;

    log::info!(
        "Built successfully, built project size is {}",
        output.project_size()
    );
    Ok(())
}

//setup a build to run wranglerjs, return the command, the ipc temp file, and the bundle
fn setup_build(target: &Target) -> Result<(Command, PathBuf, Bundle)> {
    for tool in &["node", "npm"] {
        env_dep_installed(tool)?;
    }

    let package_dir = target.package_dir()?;

    if let Some(site) = &target.site {
        site.scaffold_worker()?;
    }

    run_npm_install(&package_dir).expect("could not run `npm install`");

    let node = which::which("node").unwrap();

    let mut command = Command::new(node);

    use_legacy_openssl_if_necessary(&mut command)?;

    let wranglerjs_path = install().expect("could not install wranglerjs");
    command.arg(wranglerjs_path);

    // create a temp file for IPC with the wranglerjs process
    let mut temp_file = env::temp_dir();
    temp_file.push(format!(".wranglerjs_output{}", random_chars(5)));
    File::create(temp_file.clone())?;

    command.arg(format!(
        "--output-file={}",
        temp_file.to_str().unwrap().to_string()
    ));

    let bundle = Bundle::new(&package_dir);

    command.arg(format!("--wasm-binding={}", bundle.get_wasm_binding()));

    let custom_webpack_config_path = match &target.webpack_config {
        Some(webpack_config) => Some(PathBuf::from(&webpack_config)),
        None => {
            let config_path = PathBuf::from("webpack.config.js".to_string());
            if config_path.exists() {
                StdOut::warn("If you would like to use your own custom webpack configuration, you will need to add this to your configuration file:\nwebpack_config = \"webpack.config.js\"");
            }
            None
        }
    };

    // if webpack_config is not configured in the manifest
    // we infer the entry based on {package.json} and pass it to {wranglerjs}
    if let Some(webpack_config_path) = custom_webpack_config_path {
        build_with_custom_webpack(&mut command, &webpack_config_path);
    } else {
        build_with_default_webpack(&mut command, &package_dir)?;
    }

    Ok((command, temp_file, bundle))
}

fn build_with_custom_webpack(command: &mut Command, webpack_config_path: &Path) {
    command.arg(format!(
        "--webpack-config={}",
        &webpack_config_path.to_str().unwrap().to_string()
    ));
}

fn build_with_default_webpack(command: &mut Command, package_dir: &Path) -> Result<()> {
    let package = Package::new(package_dir)?;
    let package_main = package_dir
        .join(package.main(package_dir)?)
        .to_str()
        .unwrap()
        .to_string();
    command.arg("--no-webpack-config=1");
    command.arg(format!("--use-entry={}", package_main));
    Ok(())
}

// Run {npm install} in the specified directory. Skips the install if a
// {node_modules} is found in the directory.
fn run_npm_install(dir: &Path) -> Result<()> {
    let flock_path = dir.join(&".install.lock");
    let flock = File::create(&flock_path)?;
    // avoid running multiple {npm install} at the same time (eg. in tests)
    flock.lock_exclusive()?;

    if !dir.join("node_modules").exists() {
        let mut command = build_npm_command();
        command.current_dir(dir.to_path_buf());
        command.arg("install");
        log::info!("Running {:?} in directory {:?}", command, dir);

        let status = command.status()?;

        if !status.success() {
            anyhow::bail!("failed to execute `{:?}`: exited with {}", command, status)
        }
    } else {
        log::info!("skipping npm install because node_modules exists");
    }

    // TODO: (sven) figure out why the file doesn't exist in some cases
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
fn env_dep_installed(tool: &str) -> Result<()> {
    if which::which(tool).is_err() {
        anyhow::bail!("You need to install {}", tool)
    }
    Ok(())
}

// Install {wranglerjs} from our GitHub releases
fn install() -> Result<PathBuf> {
    let wranglerjs_path = if install::target::DEBUG {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let wranglerjs_path = source_path.join("wranglerjs");
        log::info!("wranglerjs at: {:?}", wranglerjs_path);
        wranglerjs_path
    } else {
        let tool_name = "wranglerjs";
        let tool_author = "cloudflare";
        let is_binary = false;
        let version = Version::parse(env!("CARGO_PKG_VERSION"))?;
        let wranglerjs_path = install::install(tool_name, tool_author, is_binary, version)?;
        log::info!("wranglerjs downloaded at: {:?}", wranglerjs_path.path());
        wranglerjs_path.path()
    };

    run_npm_install(&wranglerjs_path).expect("could not install wranglerjs dependencies");
    Ok(wranglerjs_path)
}

fn random_chars(n: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(n)
        .collect()
}

// If user is on Node 17+, we may need legacy OpenSSL because Webpack 4 relies on
// calls that were removed in OpenSSL 3. See:
// https://github.com/cloudflare/wrangler-legacy/issues/2108
// https://github.com/nodejs/node/blob/master/doc/changelogs/CHANGELOG_V17.md#openssl-30
// Node 17+ can still be built against OpenSSL 1, in which case the option
// doesn't exist. We need to check for that as well. See:
// https://github.com/cloudflare/wrangler-legacy/issues/2155
fn use_legacy_openssl_if_necessary(command: &mut Command) -> Result<()> {
    let node = which::which("node").unwrap();

    let mut version_check_command = Command::new(&node);
    version_check_command.arg("--version");
    let result = version_check_command.output()?.stdout;
    let need_legacy_openssl = String::from_utf8_lossy(&result)[1..3]
        .parse::<i32>()
        .unwrap()
        >= 17;

    let mut option_exists_command = Command::new(&node);
    option_exists_command.arg("--help");
    let result = option_exists_command.output()?.stdout;
    let option_exists = String::from_utf8_lossy(&result).contains("--openssl-legacy-provider");

    if need_legacy_openssl && option_exists {
        command.arg("--openssl-legacy-provider");
    }

    Ok(())
}
