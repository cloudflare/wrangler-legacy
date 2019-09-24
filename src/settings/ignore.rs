use std::fs;
use std::io::Write;
use std::io::LineWriter;

const DEFAULT_IGNORE_FILES: &[&str] = &[".*", "node_modules"];
pub const WRANGLER_IGNORE: &str = ".wranglerignore";

pub fn write_default_wranglerignore() -> Result<(), failure::Error> {
    let file = fs::File::create(WRANGLER_IGNORE)?;
    let mut file = LineWriter::new(file);

    for expression in DEFAULT_IGNORE_FILES {
        file.write(format!("{}\n", expression).as_bytes())?;
    }
    file.flush()?;

    Ok(())
}