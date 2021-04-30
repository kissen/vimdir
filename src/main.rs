mod dirops;
mod keyedbag;

use anyhow::{bail, Error};
use keyedbag::KeyedBag;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process;
use std::vec::Vec;
use structopt::StructOpt;
use tempdir::TempDir;

type DirState = Vec<PathBuf>;
type Instructions = KeyedBag<usize, PathBuf>;

fn argv0() -> String {
    let argv: Vec<String> = env::args().collect();
    return argv.first().unwrap().clone();
}

fn print_error(what: &Error) {
    eprintln!("{}: error: {}", argv0(), what);
}

fn parse_instruction_file_at(txt_file: &PathBuf) -> Result<Instructions, Error> {
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

        instructions.insert(&idx, &PathBuf::from(&new_name));
    }

    Ok(instructions)
}

fn run_editor_on(txt_file: &PathBuf) -> Result<(), Error> {
    // Get the EDITOR enviornment variable.
    let editor = env::var("EDITOR");
    if !editor.is_ok() {
        bail!("environment variable EDITOR not set");
    };

    // EDITOR can be many things, e.g.
    //
    //  "emacsclient --create-frame",
    //
    // as such we need to split it up.
    let components: Vec<String> = editor.unwrap().split(' ').map(String::from).collect();
    let executable = components.first().unwrap();
    let executable = OsStr::new(&executable);

    // Finally we can execute the command.
    let status = process::Command::new(executable)
        .args(&components[1..])
        .arg(txt_file)
        .spawn()?
        .wait()?;

    return match status.code() {
        Some(0) => Ok(()),
        Some(code) => bail!("EDITOR quit with non-zero exit code: {}", code),
        None => bail!("failed to run EDITOR"),
    };
}

fn create_instructions_file_at(script_path: &PathBuf, entries: &DirState) -> Result<(), Error> {
    let mut script_file = fs::File::create(&script_path)?;

    for (i, filename) in entries.iter().enumerate() {
        writeln!(script_file, "{}\t{}", i, filename.display())?;
    }

    Ok(())
}

fn get_files_from_working_directory() -> Result<DirState, Error> {
    let dir = env::current_dir()?;
    get_files_from_directory(&dir)
}

fn get_files_from_directory(dir: &PathBuf) -> Result<DirState, Error> {
    let mut result: DirState = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        result.push(entry.path())
    }

    Ok(result)
}

fn get_files_from_list(paths: &Vec<PathBuf>) -> Result<DirState, Error> {
    for path in paths.iter() {
        if !dirops::exists(path) {
            bail!("path does not exist: {:?}", path);
        }
    }

    Ok(paths.clone())
}

fn get_files(ops: &Opt) -> Result<DirState, Error> {
    if ops.files.is_empty() {
        return get_files_from_working_directory();
    }

    if ops.files.len() == 1 {
        let dir = &ops.files[0];
        return get_files_from_directory(dir);
    }

    get_files_from_list(&ops.files)
}

fn apply_deletes_from(instr: &Instructions, old: &DirState, ops: &Opt) -> Result<usize, Error> {
    let mut ndeleted: usize = 0;

    for (i, filepath) in old.iter().enumerate() {
        if instr.get(&i).is_none() {
            dirops::unlink(filepath, ops.recursive)?;
            ndeleted += 1;
        }
    }

    Ok(ndeleted)
}

fn first_from<T>(set: HashSet<T>) -> Option<T> {
    for elem in set {
        return Some(elem);
    }

    None
}

fn apply_copies_from(instr: &Instructions, old: &DirState, ops: &Opt) -> Result<usize, Error> {
    let mut ncopied: usize = 0;

    for key in instr.keys() {
        let old_file_name = &old[key];
        let new_file_names = instr.get(&key).unwrap();

        // If there is no new file name for the given old file, then the
        // file should be deleted (by another function).
        if new_file_names.len() == 0 {
            continue;
        }

        // If there is exactly one new file name, we can move it instead of
        // copying it.
        if new_file_names.len() == 1 {
            let new_file_name = first_from(new_file_names).unwrap();
            dirops::mv(&old_file_name, &new_file_name)?;
            continue;
        }

        // There are at least two new filenames. We have to copy.
        for new_file_name in &new_file_names {
            dirops::copy(&old_file_name, &new_file_name)?;
            ncopied += 1;
        }

        if !new_file_names.contains(old_file_name) {
            dirops::unlink(&old_file_name, ops.recursive)?;
        }
    }

    Ok(ncopied)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "vimdir")]
struct Opt {
    #[structopt(short, long, help = "Also remove directories recursively")]
    recursive: bool,

    #[structopt(name = "FILE", parse(from_os_str), help = "Files to edit")]
    files: Vec<PathBuf>,
}

fn vimdir() -> Result<(), Error> {
    // Pull a list of all entries in the directory.
    let ops = Opt::from_args();
    let entries: DirState = get_files(&ops)?;

    // Create the temporary directoy and file that is to be edited.
    let script_dir = TempDir::new("vimdir")?;
    let script_path = script_dir.path().join("instructions.txt");
    create_instructions_file_at(&script_path, &entries)?;

    // Open file in editor, let user edit the file. If the user did not modify
    // the file, do not continue.
    run_editor_on(&script_path)?;

    // Load the instructions file. Apply them.
    let instructions = parse_instruction_file_at(&script_path)?;
    apply_deletes_from(&instructions, &entries, &ops)?;
    apply_copies_from(&instructions, &entries, &ops)?;

    Ok(())
}

fn main() {
    if let Err(what) = vimdir() {
        print_error(&what);
        process::exit(1);
    }
}
