use std::{ffi::OsStr, path::PathBuf};

use bytesize::ByteSize;

/// Check a given file and ensure it's below a given size
pub fn check_file_size(file: &PathBuf, max_size: ByteSize) -> Result<(), failure::Error> {
    if !file.is_file() {
        failure::bail!("{:?} is not a file!", file)
    }
    let file_size = ByteSize::b(file.metadata()?.len());
    if file_size > max_size {
        Err(failure::format_err!(
            "{:?} is {}, which exceeds the {} limit!",
            file.file_name().unwrap_or_else(|| OsStr::new("file")),
            file_size,
            max_size
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::check_file_size;
    use bytesize::{ByteSize, MB};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::{convert::TryInto, io::Write};

    #[test]
    fn its_ok_with_small_files() -> Result<(), failure::Error> {
        let file = tempfile::NamedTempFile::new()?;
        let path_buf = file.path().to_path_buf();

        assert!(check_file_size(&path_buf, ByteSize::mb(1)).is_ok());

        Ok(())
    }

    #[test]
    fn it_errors_when_file_is_too_big() -> Result<(), failure::Error> {
        let mut file = tempfile::NamedTempFile::new()?;

        let data = thread_rng()
            .sample_iter(&Alphanumeric)
            .take((2 * MB).try_into().unwrap())
            .collect::<String>();

        writeln!(file, "{}", data)?;

        let path_buf = file.path().to_path_buf();

        assert!(check_file_size(&path_buf, ByteSize::mb(1)).is_err());

        Ok(())
    }
}
