use crate::dirops;
use crate::editor;
use crate::keyedbag::KeyedBag;
use anyhow::{bail, Error};
use git_version::git_version;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::vec::Vec;
use structopt::StructOpt;
use tempdir::TempDir;

#[allow(dead_code)]
const VIMDIR_VERSION: &'static str = env!("CARGO_PKG_VERSION");

type Instructions = KeyedBag<usize, PathBuf>;

struct DirState {
    entries: Vec<PathBuf>,
    parent: PathBuf,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "vimdir", version=git_version!(fallback=VIMDIR_VERSION))]
struct Opt {
    #[structopt(short, long, help = "Also remove directories recursively")]
    recursive: bool,

    #[structopt(
        short,
        long,
        help = "Verbosely display the actions taken by the program"
    )]
    verbose: bool,

    #[structopt(short, long, help = "Ignore hidden files")]
    ignore_hidden_files: bool,

    #[structopt(name = "FILE", parse(from_os_str), help = "Files to edit")]
    files: Vec<PathBuf>,
}

/// Return the first arg as passed to the program. Usually,
/// this is the name of the executable.
fn argv0() -> String {
    env::args().next().unwrap()
}

/// Read instruction file at "txt_file" and return the individual items.
fn parse_instruction_file_at(txt_file: &PathBuf, state: &DirState) -> Result<Instructions, Error> {
    let file = fs::File::open(txt_file)?;
    let mut instructions: Instructions = KeyedBag::new();
    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let components: Vec<String> = line.splitn(2, "\t").map(String::from).collect();

        if components.len() != 2 {
            bail!("bad line: {}", line);
        }

        let idx_string: &String = &components[0];
        let new_name: &String = &components[1];

        let idx: usize = match idx_string.parse() {
            Ok(value) => value,
            Err(_) => bail!("bad index: {}", idx_string),
        };

        let new_path = state.parent.join(PathBuf::from(&new_name));
        instructions.insert(idx, new_path);
    }

    Ok(instructions)
}

/// Create the template instruction file, containing "state.entries". The
/// file will be written to "path".
fn create_instructions_file_at(path: &PathBuf, state: &DirState) -> Result<(), Error> {
    let mut file = dirops::open_for_user(path)?;

    for (i, filepath) in state.entries.iter().enumerate() {
        let filename = dirops::file_name(filepath)?;
        writeln!(file, "{}\t{}", i, filename.display())?;
    }

    Ok(())
}

/// Check whether "path" actually exists. If it does not, throw an error.
fn ensure_paths_exists(path: &PathBuf) -> Result<(), Error> {
    if !path.exists() {
        bail!("path does not exist: {}", path.display());
    }

    Ok(())
}

/// Check whether "path" has "expected_parent" as parent. Throw an error
/// if it does not.
fn ensure_parent_matches(path: &PathBuf, expected_parent: &PathBuf) -> Result<(), Error> {
    let path_parent = dirops::parent(path)?;

    if &path_parent != expected_parent {
        bail!(
            "directories not unique: {} and {}",
            expected_parent.display(),
            path_parent.display()
        );
    }

    Ok(())
}

/// Get a file listing from the current working directory.
fn get_files_from_working_directory(ignore_hidden: bool) -> Result<DirState, Error> {
    let dir = env::current_dir()?;
    get_files_from_directory(&dir, ignore_hidden)
}

/// Get a file listing from directory "dir".
fn get_files_from_directory(dir: &PathBuf, ignore_hidden: bool) -> Result<DirState, Error> {
    let mut result: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if ignore_hidden && dirops::is_hidden(&path) {
            continue;
        }

        result.push(path);
    }

    let state = DirState {
        entries: result,
        parent: dir.clone(),
    };

    Ok(state)
}

/// Get a file listing containing each element in "paths". This function
/// fails when any of the elements in "paths" do not exist.
fn get_files_from_list(paths: &Vec<PathBuf>, ignore_hidden: bool) -> Result<DirState, Error> {
    // We require that all files have the same parent path. Doing
    // things different gets confusing very quickly. Check that here.

    let expected_parent = dirops::parent(paths.first().unwrap())?;
    let mut cleaned: Vec<PathBuf> = Vec::new();

    for path in paths.iter() {
        ensure_paths_exists(&path)?;
        ensure_parent_matches(&path, &expected_parent)?;

        if ignore_hidden && dirops::is_hidden(&path) {
            continue;
        }

        cleaned.push(path.clone());
    }

    let state = DirState {
        entries: cleaned,
        parent: expected_parent,
    };

    Ok(state)
}

/// Get the listing of files and directories to edit.
fn get_files(ops: &Opt) -> Result<DirState, Error> {
    let ignore_hidden = ops.ignore_hidden_files;

    let mut state: DirState = if ops.files.is_empty() {
        // If we have no files supplied by the user, run vimdir
        // on the current working directory.
        get_files_from_working_directory(ignore_hidden)?
    } else if ops.files.len() == 1 && ops.files.first().unwrap().is_dir() {
        // Because we only have exactly one entry at it appears to
        // be a directory, treat it as such. This does introduce a
        // race condition as the check about and reading the dir
        // in the call below are two separate actions. Oh no!
        let name = &ops.files[0];
        get_files_from_directory(name, ignore_hidden)?
    } else {
        // We have some number of entries. At this point, we only
        // support multiple files, not multiple (opened)
        // directories.
        get_files_from_list(&ops.files, ignore_hidden)?
    };

    if state.entries.is_empty() {
        bail!("no files to edit");
    }

    state.entries.sort_unstable();
    Ok(state)
}

/// Delete those files that previously were in "old" but are now
/// missing from "instr".
fn apply_deletes_from(instr: &Instructions, old: &DirState, ops: &Opt) -> Result<usize, Error> {
    let mut ndeleted: usize = 0;

    for (i, filepath) in old.entries.iter().enumerate() {
        if instr.get(&i).is_empty() {
            if ops.verbose {
                println!("{}: rm {:?}", argv0(), filepath);
            }

            dirops::unlink(filepath, ops.recursive)?;
            ndeleted += 1;
        }
    }

    Ok(ndeleted)
}

/// Return some element from "set". Because sets have no ordering,
/// what value in set "set" you get has to considered random.
/// This function is useful when "set" only contains one element
/// and you want to get that one element.
fn first_from<T>(set: &HashSet<T>) -> Option<&T> {
    for elem in set {
        return Some(elem);
    }

    None
}

/// Duplicate files as instructed by "instr".
fn apply_copies_from(instr: &Instructions, old: &DirState, ops: &Opt) -> Result<usize, Error> {
    let mut ncopied: usize = 0;

    for key in instr.keys() {
        let old_file_name = &old.entries[*key];
        let new_file_names = instr.get(&key);

        // If there is no new file name for the given old file, then the
        // file should be deleted (by another function).
        if new_file_names.len() == 0 {
            continue;
        }

        // If there is exactly one new file name, we can move it instead of
        // copying it.
        if new_file_names.len() == 1 {
            let new_file_name = first_from(new_file_names).unwrap();

            if ops.verbose {
                println!("{}: mv {:?} {:?}", argv0(), old_file_name, new_file_name);
            }

            dirops::mv(&old_file_name, &new_file_name)?;

            continue;
        }

        // There are at least two new filenames. We have to copy.
        for new_file_name in new_file_names {
            if ops.verbose {
                println!("{}: cp {:?} {:?}", argv0(), old_file_name, new_file_name);
            }

            dirops::copy(&old_file_name, &new_file_name)?;
            ncopied += 1;
        }

        if !new_file_names.contains(old_file_name) {
            if ops.verbose {
                println!("{}: rm {:?}", argv0(), old_file_name);
            }

            dirops::unlink(&old_file_name, ops.recursive)?;
        }
    }

    Ok(ncopied)
}

pub fn run() -> Result<(), Error> {
    // Pull a list of all entries in the directory.
    let ops = Opt::from_args();
    let state: DirState = get_files(&ops)?;

    // Create the temporary directory and file that is to be edited.
    let script_dir = TempDir::new("vimdir")?;
    let script_path = script_dir.path().join("instructions.txt");
    create_instructions_file_at(&script_path, &state)?;

    // Open file in editor, let user edit the file. If the user did not modify
    // the file, do not continue.
    editor::run_on(&script_path)?;

    // Load the instructions file. Apply them.
    let instructions = parse_instruction_file_at(&script_path, &state)?;
    apply_deletes_from(&instructions, &state, &ops)?;
    apply_copies_from(&instructions, &state, &ops)?;

    Ok(())
}
