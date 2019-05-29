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
