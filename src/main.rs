//! CLI program to show the changes in a Taskwarrior database.

use std::env;
use std::ffi::OsString;
use std::io;

mod changes;
mod pager;
mod taskdb;

/// Default path for the Taskwarrior command.
const DEFAULT_TASK_PATH: &str = "task";

/// Environment variable to use a different command.
const TASK_PATH_ENV: &str = "TASKWARRIOR_PATH";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let changes_limit = get_changes_limit()?;
    let taskdb = taskdb::TaskDb::new(&task_command())?;

    let stdout_handle;
    let pager_command;
    let mut output: Box<dyn io::Write>;

    match pager::command() {
        None => {
            pager_command = None;
            stdout_handle = io::stdout();
            output = Box::new(io::BufWriter::new(stdout_handle.lock()));
        }

        Some(mut command) => {
            let mut command = command.spawn()?;
            output = Box::new(command.stdin.take().unwrap());
            pager_command = Some(command);
        }
    }

    for change in taskdb.changes.iter().rev().take(changes_limit) {
        if changes::show(&taskdb, &change, &mut output).is_err() {
            break;
        }
    }

    drop(output);
    if let Some(mut command) = pager_command {
        let _ = command.wait();
    }

    Ok(())
}

/// Find path to execute the Taskwarrior command.
fn task_command() -> OsString {
    env::var_os(TASK_PATH_ENV).unwrap_or_else(|| OsString::from(DEFAULT_TASK_PATH))
}

/// Get number of changes to show from the command line.
fn get_changes_limit() -> Result<usize, &'static str> {
    let mut args = env::args().skip(1).map(|a| a.parse());
    match (args.next(), args.next()) {
        (None, None) => Ok(usize::MAX),
        (Some(Ok(n)), None) => Ok(n),
        _ => Err("Invalid arguments. Usage: task-change [count]"),
    }
}
