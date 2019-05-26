use crate::commands::publish::package::Package;
use log::info;
use std::env;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

mod bundle;

pub use bundle::Bundle;

mod output;

use output::WranglerjsOutput;

// Run the underlying {wrangler-js} executable.
//
// In Rust we create a virtual file, pass the pass to {wrangler-js}, run the
// executable and wait for completion. The file will receive the a serialized
// {WranglerjsOutput} struct.
// Note that the ability to pass a fd is platform-specific
pub fn run_build(
    wasm_pack_path: PathBuf,
    bundle: &Bundle,
) -> Result<WranglerjsOutput, failure::Error> {
    install()?;
    let mut command = Command::new("wrangler-js");
    command.env("WASM_PACK_PATH", wasm_pack_path);

    // create temp file for special {wrangler-js} IPC.
    let mut temp_file = env::temp_dir();
    temp_file.push(".wranglerjs_output");
    File::create(temp_file.clone())?;

    command.arg(format!(
        "--output-file={}",
        temp_file.clone().to_str().unwrap().to_string()
    ));
    command.arg(format!("--wasm-binding={}", bundle.get_wasm_binding()));

    // if {webpack.config.js} is not present, we infer the entry based on the
    // {package.json} file and pass it to {wrangler-js}.
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

pub fn run_npm_install() -> Result<(), failure::Error> {
    let mut command = build_npm_command();

    command.arg("install");
    info!("Running {:?}", command);

    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        failure::bail!("failed to execute `{:?}`: exited with {}", command, status)
    }
}

fn env_dep_installed(tool: &str) -> Result<(), failure::Error> {
    if which::which(tool).is_err() {
        failure::bail!("You need to install {}", tool)
    }
    Ok(())
}

pub fn install() -> Result<(), failure::Error> {
    for tool in &["node", "npm"] {
        env_dep_installed(tool)?;
    }

    if which::which("wrangler-js").is_err() {
        let mut command = build_npm_command();
        command
            .arg("install")
            .arg("https://github.com/ashleygwilliams/wrangler-js")
            .arg("-g");
        info!("Running {:?}", command);

        let status = command.status()?;
        if !status.success() {
            failure::bail!("failed to execute `{:?}`: exited with {}", command, status)
        }
    }
    Ok(())
}

/// build a Command for npm
///
/// Here's the deal: on Windows, `npm` isn't a binary, it's a shell script.
/// This means that we can't invoke it via `Command` directly on Windows,
/// we need to invoke `cmd /C npm`, to run it within the cmd environment.
fn build_npm_command() -> Command {
    #[cfg(not(windows))]
    let command = Command::new("npm");

    #[cfg(windows)]
    let mut command = Command::new("cmd");
    #[cfg(windows)]
    command.arg("/C");
    #[cfg(windows)]
    command.arg("npm");

    command
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
