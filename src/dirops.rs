use anyhow::{bail, Error};
use fs_extra;
use std::fs;
use std::path::PathBuf;

pub fn exists(path: &PathBuf) -> bool {
    fs::metadata(path).is_ok()
}

pub fn is_dir(path: &PathBuf) -> bool {
    match fs::metadata(path) {
        Ok(meta) => meta.is_dir(),
        Err(_) => false,
    }
}

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

pub fn copy(from: &PathBuf, to: &PathBuf) -> Result<(), Error> {
    // When from and to are the same, there is nothing to do. We
    // can return w/o any io operations.

    if from == to {
        return Ok(());
    }

    // Paths from and to are actually different. We will need to do
    // some work.

    let meta = fs::metadata(from)?;

    if meta.is_dir() {
        let config = fs_extra::dir::CopyOptions::new();
        fs_extra::dir::copy(from, to, &config)?;
    } else if meta.is_file() {
        let config = fs_extra::file::CopyOptions::new();
        fs_extra::file::copy(from, to, &config)?;
    } else {
        bail!("bad file type");
    }

    Ok(())
}

pub fn mv(from: &PathBuf, to: &PathBuf) -> Result<(), Error> {
    if from == to {
        return Ok(());
    }

    let meta = fs::metadata(from)?;

    if meta.is_dir() {
        let config = fs_extra::dir::CopyOptions::new();
        fs_extra::dir::move_dir(from, to, &config)?;
    } else if meta.is_file() {
        let config = fs_extra::file::CopyOptions::new();
        fs_extra::file::move_file(from, to, &config)?;
    } else {
        bail!("bad file type");
    }

    Ok(())
}
