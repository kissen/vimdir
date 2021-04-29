use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, Error, ErrorKind, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::vec::Vec;
use tempdir::TempDir;

type Instructions = HashMap<u64, PathBuf>;

fn argv0() -> String {
    let argv: Vec<String> = env::args().collect();
    return argv.first().unwrap().clone();
}

fn print_error(what: Error) {
    eprintln!("{}: error: {}", argv0(), what);
}

fn err(message: &str) -> Result<(), Error> {
    Err(Error::new(ErrorKind::Other, message))
}

fn parse_instruction_file_at(txt_file:&PathBuf) -> Result<Instructions, io::Error> {
    let file = fs::File::open(txt_file)?;
    let mut instructions: Instructions = HashMap::new();

    for line in io::BufReader::new(file).lines() {
        let line = line?;
        let components: Vec<String> = line.splitn(2, "\t").map(String::from).collect();

        if components.len() != 2 {
            let what: String = format!("bad line{}", line);
            return Err(Error::new(ErrorKind::Other, what));
        }

        let idx_string: &String = &components[0];
        let new_name: &String = &components[1];

        let idx: u64 = match idx_string.parse() {
            Ok(value) => value,
            Err(_) => return Err(Error::new(ErrorKind::Other, "bad index"))
        };

        instructions.insert(idx, PathBuf::from(&new_name));
    }

    Ok(instructions)
}

fn run_editor_on(txt_file: &PathBuf) -> Result<process::ExitStatus, Error> {
    // Get the EDITOR enviornment variable.
    let editor = env::var("EDITOR");
    if !editor.is_ok() {
        return Err(Error::new(ErrorKind::NotFound, "environment variable EDITOR not set"))
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
    process::Command::new(executable).args(&components[1..]).arg(txt_file).spawn()?.wait()
}

fn vimdir() -> Result<(), Error> {
    let dir = Path::new("/mnt/ramdisk/");
    // Pull a list of all entries in the directory.
    let entries = get_files(dir)?;

    // Create the temporary directoy and file that is to be edited.
    let script_dir = TempDir::new("vimdir")?;
    let script_path = script_dir.path().join("instructions.txt");

    {
        // Create the instructions file. We do this in an extra scope so
        // the file gets closed once we left this block, i.e. it is ready
        // for editing by the user.
        let mut script_file = fs::File::create(&script_path)?;

        for (i, filename) in entries.iter().enumerate() {
            writeln!(script_file, "{}\t{}", i, filename.display())?;
        }
    }

    // Open file in editor, let user edit the file. If the user did not modify
    // the file, do not continue.
    let exit_code = run_editor_on(&script_path)?.code().unwrap_or(1);
    if exit_code != 0 {
        return err("non-zero exit code");
    }

    // Load the instructions file.
    let instructions = parse_instruction_file_at(&script_path)?;

    for (id, name) in &instructions {
        println!("{}: {:?}", id, name);
    }

    Ok(())
}

fn main() {
    let result = vimdir();

    if !result.is_ok() {
        print_error(result.err().unwrap());
        process::exit(1);
    }

    println!("ok");
}

fn get_files(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut result: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        result.push(entry.path())
    }

    Ok(result)
}
