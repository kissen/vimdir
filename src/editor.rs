use std::ffi::OsStr;
use std::path::PathBuf;

use anyhow::{bail, Error};
use std::env;
use std::process;

/// Get the name of the editor. We look at multiple environment variables
/// to achieve this. If no variable is set, we fall back to "vi" which is
/// installed on every reasonable system.
fn get_editor_command() -> String {
    let variables = vec!["EDITOR", "VISUAL"];

    for variable in variables {
        if let Ok(editor) = env::var(variable) {
            return editor;
        }
    }

    return String::from("vi");
}

/// Spawn the default editor to edit file at "txt_file".
pub fn run_on(txt_file: &PathBuf) -> Result<(), Error> {
    // EDITOR can be many things, e.g.
    //
    //  "emacsclient --create-frame",
    //
    // as such we need to split it up.
    let editor = get_editor_command();
    let components: Vec<String> = editor.split(' ').map(String::from).collect();
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
