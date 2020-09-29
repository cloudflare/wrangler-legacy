use std::path::PathBuf;

use super::check_file_size;

// // i'm not sure whether it's better to do a pointer to an array of &str, or specify the length.
// // my gut feeling is that specifying the length is better since the lengths are known at compile time
// const UNAVAILABLE_BUILTINS: [&str; 2] = ["eval", "new Function"];
// const AVAILABLE_BUILTINS: [&str; 5] = ["atob", "btoa", "TextEncoder", "TextDecoder", "URL"];
// const AVAILABLE_WITHIN_REQUEST_CONTEXT: [&str; 5] = [
//     "setInterval",
//     "clearInterval",
//     "setTimeout",
//     "clearTimeout",
//     "fetch",
// ];

pub async fn check_js(
    js_file: &PathBuf,
    _sourcemap_file: Option<&PathBuf>,
) -> Result<String, failure::Error> {
    // these checks don't operate on the AST, so we just run them right away
    let file_size = check_file_size(js_file)?;

    Ok(format!("worker.js OK! Final size: {}", file_size))
}
