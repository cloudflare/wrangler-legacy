use crate::commands::DEFAULT_CONFIG_PATH;
use crate::settings::{self, toml::Manifest};

use std::collections::HashMap;
use std::env::args;
use std::io::Write;
use std::panic;
use std::panic::PanicInfo;
use std::path::Path;

use backtrace::{Backtrace, BacktraceFrame};
use path_slash::PathExt;
use serde::Serialize;
use sys_info::{os_release, os_type};
use uuid::Uuid;

const PANIC_UNWIND_START_MARKER: &str = "rust_begin_unwind";
const BACKTRACE_PATH_PREFIX: &str = "backtrace::";

#[derive(Debug, Serialize)]
struct Report {
    uuid: Uuid,
    timestamp_ms: u128,
    host_env: HashMap<&'static str, String>, // "os": "..." TODO: consider struct over HashMaps
    project_info: HashMap<&'static str, String>,
    args: Vec<String>,
    panic: Option<String>,
    location: Option<String>,
    backtrace: String,
}

/// Overrides any panic hooks with wrangler's error reporting, which logs error reports to disl with
/// details from a panic and useful information from the wrangler user's system for debugging.
pub fn init() {
    // TODO: consider using panic::take_hook, and showing the original panic to the end-user without
    // polluting the console to the point the wrangler error report message is lost in the noise.
    panic::set_hook(Box::new(|panic_info| {
        // gather necessary error report information, to be stored on disk until uploaded
        let mut report = Report {
            uuid: Uuid::new_v4(),
            timestamp_ms: 0,
            host_env: load_host_info(),
            project_info: load_project_info(),
            args: args().collect::<Vec<_>>(),
            panic: panic_payload(&panic_info),
            location: None,
            backtrace: useful_frames(),
        };

        if let Some(loc) = panic_info.location() {
            report.location = Some(format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));
        }

        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(ts) => {
                report.timestamp_ms = ts.as_millis();
            }
            Err(e) => {
                err_exit(format!("system time failure: {}", e), 1);
            }
        }

        // write the report to disk using a timestamp-like name
        let wrangler_error_dir = match settings::get_wrangler_home_dir() {
            Ok(dir) => dir.join(Path::new("errors")),
            Err(e) => {
                err_exit(format!("directory location error: {}", e), 1);
                unreachable!()
            }
        };

        if let Ok(data) = serde_json::to_string_pretty(&report) {
            if std::fs::create_dir_all(wrangler_error_dir.clone()).is_err() {
                err_exit("failed to create report directory", 1);
            }

            let mut file = std::fs::File::create(
                wrangler_error_dir.join(format!("{}.log", report.timestamp_ms)),
            )
            .unwrap_or_else(|_| {
                err_exit("failed to create report file", 1);
                unreachable!()
            });

            if file.write_all(data.as_bytes()).is_err() {
                err_exit("failed to write report", 1);
            }
        } else {
            err_exit(format!("Wrangler encountered an unrecoverable error and failed to write the report: \n{:#?}", report), 1);
        }

        // print message to user with note about the crash and how to report it using the command
        // `wrangler report <timestamp-name>`
        eprintln!(
            "{}", 
            &format!(
                "Oops! Wrangler hit a snag... please help us debug the issue by submitting the generated error report ({})\n", 
                wrangler_error_dir.join(format!("{}.log", report.timestamp_ms)).to_slash_lossy()
            ),
        );
        eprintln!("To submit this error report to the Wrangler team now, run:");
        eprintln!("\n\t$ wrangler report\n");
    }));
}

// host system information
fn load_host_info() -> HashMap<&'static str, String> {
    let mut host_info = HashMap::new();
    if let Ok(release) = os_release() {
        host_info.insert("os_release", release);
    }
    if let Ok(typ) = os_type() {
        host_info.insert("os_type", typ);
    }

    if let Ok(version) = os_version::detect() {
        host_info.insert("os_version", version.to_string());
    }

    host_info.insert(
        "wrangler_version",
        option_env!("CARGO_PKG_VERSION")
            .unwrap_or_else(|| "unknown")
            .into(),
    );

    host_info
}

// wrangler project information
fn load_project_info() -> HashMap<&'static str, String> {
    let mut project_info = HashMap::new();

    if let Ok(manifest) = Manifest::new(Path::new(DEFAULT_CONFIG_PATH)) {
        project_info.insert("script_name", manifest.name);
        project_info.insert("account_id", manifest.account_id);
        project_info.insert("zone_id", manifest.zone_id.unwrap_or_else(|| "".into()));
        project_info.insert("target_type", manifest.target_type.to_string());
        project_info.insert(
            "workers_dev",
            manifest.workers_dev.unwrap_or(false).to_string(),
        );
        if let Some(builder) = manifest.build {
            project_info.insert("using_custom_build", "true".into());

            if let Some((command, _)) = builder.build_command() {
                project_info.insert("custom_build_command", command.into());
            }

            // TODO: encode the format's struct members in map field instead of only string literal
            project_info.insert(
                "upload_format",
                match builder.upload {
                    settings::toml::UploadFormat::ServiceWorker { .. } => "service-worker".into(),
                    settings::toml::UploadFormat::Modules { .. } => "modules".into(),
                },
            );
        }

        if let Some(routes) = manifest.routes {
            project_info.insert("routes", routes.join(","));
        }

        if let Some(route) = manifest.route {
            project_info.insert("route", route);
        }

        if let Some(usage_model) = manifest.usage_model {
            project_info.insert("usage_model", usage_model.as_ref().into());
        }
    }

    project_info
}

// removes frames before wrangler takes over at the panic, reduces noise
fn useful_frames() -> String {
    let bt = Backtrace::new();
    let frames = bt.frames();
    let found_idx = frames.iter().position(|frame| {
        for sym in frame.symbols() {
            if let Some(name) = sym.name() {
                return name.to_string() == PANIC_UNWIND_START_MARKER;
            }
        }

        false
    });
    let skip_count = match found_idx {
        Some(idx) => idx + 1,
        None => 0,
    };

    let useful = frames
        .iter()
        .skip(skip_count)
        .filter(|frame| {
            for sym in frame.symbols() {
                if let Some(name) = sym.name() {
                    return !name.to_string().starts_with(BACKTRACE_PATH_PREFIX);
                }
            }

            true
        })
        .cloned()
        .collect::<Vec<BacktraceFrame>>();

    format!("{:?}", Backtrace::from(useful)).trim().to_string()
}

// extracts the payload contents from the panic (e.g. panic!("this is the payload"))
// TODO: consider other <T> for downcast_ref and handle
fn panic_payload(info: &PanicInfo) -> Option<String> {
    match info.payload().downcast_ref::<&str>() {
        Some(s) => Some(s.to_string()),
        None => None,
    }
}

fn err_exit<S: AsRef<str>>(msg: S, code: i32) {
    eprintln!("wrangler panic handle error: {}", msg.as_ref());
    std::process::exit(code);
}
