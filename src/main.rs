mod dirops;
mod editor;
mod keyedbag;
mod vimdir;

use anyhow::Error;
use std::env;
use std::process;

/// Print error "what" to stderr.
fn print_error(what: &Error) {
    let cmd = env::args().next().unwrap();
    eprintln!("{}: error: {}", cmd, what);
}

fn main() {
    if let Err(what) = vimdir::run() {
        print_error(&what);
        process::exit(1);
    }
}
