/// This file customizes debug builds for Wrangler
///
/// 1) sets the debug flag for rustc so Wrangler can feature detect
///    https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-cfg
/// 2) set the source directory for Wrangler so debug builds know where to
///    look for wranglerjs source files
///    https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorustc-envvarvalue
use std::env;

fn main() {
    if let Ok(profile) = env::var("PROFILE") {
        if profile == "debug" {
            println!("cargo:rustc-cfg=feature={:?}", profile);
            println!(
                "cargo:rustc-env=SOURCE_DIR={:?}",
                env::current_dir().unwrap()
            );
        } else {
            println!("cargo:rustc-env=SOURCE_DIR={:?}", "");
        }
    }
}
