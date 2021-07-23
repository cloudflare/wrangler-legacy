use crate::settings::{self, toml::Manifest};
use crate::{
    commands::DEFAULT_CONFIG_PATH,
    settings::toml::{DurableObjects, UploadFormat},
};

use std::env::args;
use std::fs::{read_dir, File};
use std::io::{BufReader, Write};
use std::panic;
use std::panic::PanicInfo;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fs};

use anyhow::{anyhow, Result};
use backtrace::{Backtrace, BacktraceFrame};
use path_slash::PathExt;
use serde::{Deserialize, Serialize};
use sys_info::{os_release, os_type};
use uuid::Uuid;

const PANIC_UNWIND_START_MARKER: &str = "rust_begin_unwind";
const BACKTRACE_PATH_PREFIX: &str = "backtrace::";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Report {
    uuid: Uuid,
    timestamp_ms: u128,
    host_env: HashMap<String, String>,
    project_info: HashMap<String, String>,
    durable_objects: Option<DurableObjects>,
    upload_format: Option<UploadFormat>,
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
    panic::set_hook(Box::new(|panic_info| generate_report(Some(panic_info))));
}

/// finds the most recent log in the error report directory, unless `log` is Some, and will try to
/// read that file instead.
pub fn read_log(log: Option<&Path>) -> Result<Report> {
    match log {
        Some(path) => {
            let r = BufReader::new(File::open(error_report_dir()?.join(path))?);
            serde_json::from_reader(r).map_err(|e| e.into())
        }
        None => latest_report(),
    }
}

/// gathers necessary error report information, and stores on disk until uploaded. the
pub fn generate_report(panic_info: Option<&PanicInfo>) {
    let project_info = load_project_info();
    let mut report = Report {
        uuid: Uuid::new_v4(),
        timestamp_ms: 0,
        host_env: load_host_info(),
        project_info: project_info.base,
        durable_objects: project_info.durable_objects,
        upload_format: project_info.upload_format,
        args: args().collect::<Vec<_>>(),
        panic: panic_info.and_then(try_extract_payload),
        location: None,
        backtrace: useful_frames(),
    };

    if let Some(info) = panic_info {
        if let Some(loc) = info.location() {
            report.location = Some(format!(
                "{}, line: {} column: {}",
                loc.file(),
                loc.line(),
                loc.column()
            ));
        }
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
    let wrangler_error_dir = error_report_dir().unwrap_or_else(|e| {
        err_exit(format!("directory location error: {}", e), 1);
    });
    if let Ok(data) = serde_json::to_string_pretty(&report) {
        if std::fs::create_dir_all(wrangler_error_dir.clone()).is_err() {
            err_exit("failed to create report directory", 1);
        }

        let mut file =
            std::fs::File::create(wrangler_error_dir.join(format!("{}.log", report.timestamp_ms)))
                .unwrap_or_else(|_| {
                    err_exit("failed to create report file", 1);
                });

        if file.write_all(data.as_bytes()).is_err() {
            err_exit("failed to write report", 1);
        }
    } else {
        err_exit(format!("wrangler encountered an unrecoverable error and failed to write the report: \n{:#?}", report), 1);
    }

    // print message to user with note about the crash and how to report it using the command
    // `wrangler report --log=<filename.log>`
    eprintln!(
        r#"
Oops! wrangler encountered an error... please help Cloudflare debug this issue by submitting an error report ({})

To submit this error report to Cloudflare, run:

    $ wrangler report
        "#,
        wrangler_error_dir
            .join(format!("{}.log", report.timestamp_ms))
            .to_slash_lossy(),
    );
}

// finds the most-recently created error report based on the timestamped filename within the
// expected directory
fn latest_report() -> Result<Report> {
    let mut files = vec![];
    for entry in read_dir(error_report_dir()?)? {
        let entry = entry?;
        let md = fs::metadata(entry.path())?;
        if md.is_file() {
            files.push(entry.path())
        }
    }
    files.sort();

    if let Some(f) = files.last() {
        let reader = BufReader::new(File::open(f)?);
        return serde_json::from_reader(reader).map_err(|e| e.into());
    }

    Err(anyhow!("no error reports found"))
}

// returns the path to the location of wrangler's error report log files
fn error_report_dir() -> Result<PathBuf> {
    let base = settings::get_wrangler_home_dir();
    let report_dir = base.join(Path::new("errors"));
    fs::create_dir_all(report_dir.clone())?;
    Ok(report_dir)
}

// host system information
fn load_host_info() -> HashMap<String, String> {
    let mut host_info = HashMap::new();

    if let Ok(release) = os_release() {
        host_info.insert("os_release".into(), release);
    }

    if let Ok(typ) = os_type() {
        host_info.insert("os_type".into(), typ);
    }

    if let Ok(version) = os_version::detect() {
        host_info.insert("os_version".into(), version.to_string());
    }

    host_info.insert(
        "wrangler_version".into(),
        option_env!("CARGO_PKG_VERSION")
            .unwrap_or_else(|| "unknown")
            .into(),
    );

    host_info
}

#[derive(Default)]
struct ProjectInfo {
    base: HashMap<String, String>,
    durable_objects: Option<DurableObjects>,
    upload_format: Option<UploadFormat>,
}

// wrangler project information
fn load_project_info() -> ProjectInfo {
    let mut project_info: ProjectInfo = Default::default();

    if let Ok(manifest) = Manifest::new(Path::new(DEFAULT_CONFIG_PATH)) {
        project_info
            .base
            .insert("script_name".into(), manifest.name);
        project_info.base.insert(
            "account_id".into(),
            manifest.account_id.maybe_load().unwrap_or_default(),
        );
        project_info.base.insert(
            "zone_id".into(),
            manifest.zone_id.unwrap_or_else(|| "".into()),
        );
        project_info
            .base
            .insert("target_type".into(), manifest.target_type.to_string());
        project_info.base.insert(
            "workers_dev".into(),
            manifest.workers_dev.unwrap_or(false).to_string(),
        );
        if let Some(builder) = manifest.build {
            project_info
                .base
                .insert("using_custom_build".into(), "true".into());

            if let Some((command, _)) = builder.build_command() {
                project_info
                    .base
                    .insert("custom_build_command".into(), command.into());
            }

            project_info.upload_format = Some(builder.upload);
        }

        if let Some(routes) = manifest.routes {
            project_info.base.insert("routes".into(), routes.join(","));
        }

        if let Some(route) = manifest.route {
            project_info.base.insert("route".into(), route);
        }

        if let Some(usage_model) = &manifest.usage_model {
            project_info
                .base
                .insert("usage_model".into(), usage_model.as_ref().to_string());
        }

        if let Some(durable_objects) = &manifest.durable_objects {
            project_info.durable_objects = Some(durable_objects.clone());
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

/// Attempts to extract the payload contents from the panic (e.g. panic!("this is the payload"))
fn try_extract_payload(panic_info: &PanicInfo) -> Option<String> {
    // This currently only handles standard panic payloads (&str and String). If we ever use
    // panic_any, we'll have to explicitly handle it here.
    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        Some(s.to_string())
    } else {
        panic_info.payload().downcast_ref::<String>().cloned()
    }
}

fn err_exit<S: AsRef<str>>(msg: S, code: i32) -> ! {
    eprintln!(
        "wrangler encountered an error while attempting to log an error report: {}",
        msg.as_ref()
    );
    std::process::exit(code);
}
