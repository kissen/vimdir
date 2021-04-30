use anyhow::{bail, Error};
use std::fs;
use std::path::PathBuf;

pub fn unlink(path: &PathBuf) -> Result<(), Error> {
    let meta = fs::metadata(path)?;

    if meta.is_dir() {
        fs::remove_dir_all(path)?
    } else if meta.is_file() {
        fs::remove_file(path)?;
    } else {
        bail!("bad file type");
    }

    Ok(())
}
