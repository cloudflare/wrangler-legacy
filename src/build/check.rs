use std::path::PathBuf;

// TODO: i'm not sure whether it's better to do a pointer to an array of &str, or specify the length
// TODO: my gut feeling is that specifying the length is better since the lengths are known at compile time
const UNAVAILABLE_BUILTINS: [&str; 2] = ["eval", "new Function"];
const AVAILABLE_BUILTINS: [&str; 5] = ["atob", "btoa", "TextEncoder", "TextDecoder", "URL"];
const AVAILABLE_WITHIN_REQUEST_CONTEXT: [&str; 5] = [
    "setInterval",
    "clearInterval",
    "setTimeout",
    "clearTimeout",
    "fetch",
];

pub fn full_check(dir: PathBuf) -> Result<String, failure::Error> {
    todo!()
}
