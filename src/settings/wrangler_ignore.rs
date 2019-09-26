use std::fs;
use std::io::LineWriter;
use std::io::Write;
use std::path::Path;

const DEFAULT_IGNORE_FILES: &[&str] = &[".*", "node_modules/"];
pub const WRANGLER_IGNORE: &str = ".wranglerignore";

pub fn write_default_wranglerignore(config_path: &Path) -> Result<(), failure::Error> {
    let file = fs::File::create(config_path.join(WRANGLER_IGNORE))?;
    let mut file = LineWriter::new(file);

    for expression in DEFAULT_IGNORE_FILES {
        file.write_all(format!("{}\n", expression).as_bytes())?;
    }
    file.flush()?;

    Ok(())
}
