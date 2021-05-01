use anyhow::{bail, Error};
use fs_extra;
use std::fs::{self, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

pub fn open_for_user(path: &PathBuf) -> Result<fs::File, Error> {
    let mut options = OpenOptions::new();
    options.create(true);
    options.write(true);
    options.mode(0o600);

    Ok(options.open(&path)?)
}

/// Delete file at path. If path is a directory, it is only deleted
/// when recursive is set to true.
pub fn unlink(path: &PathBuf, recursive: bool) -> Result<(), Error> {
    let meta = fs::metadata(path)?;

    if meta.is_dir() {
        if !recursive {
            bail!("refusing to unlink: {:?}", path);
        }
        fs::remove_dir_all(path)?
    } else if meta.is_file() {
        fs::remove_file(path)?;
    } else {
        bail!("bad file type");
    }

    Ok(())
}

/// Copy file or directory from -> to.
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

/// Move file or directory from -> to.
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
