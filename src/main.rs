//! CLI program to show the changes in a Taskwarrior database.

use std::env;
use std::ffi::OsString;
use std::io;

use clap::Parser;

mod changes;
mod pager;
mod taskdb;

/// Default path for the Taskwarrior command.
const DEFAULT_TASK_PATH: &str = "task";

/// Environment variable to use a different command.
const TASK_PATH_ENV: &str = "TASKWARRIOR_PATH";

#[derive(Parser)]
struct Args {
    /// Print changes in the short format.
    #[arg(short, long)]
    short: bool,

    /// Number of changes to print.
    changes: Option<usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let changes_limit = args.changes.unwrap_or(usize::MAX);
    let taskdb = taskdb::TaskDb::new(&task_command())?;

    let stdout_handle;
    let pager_command;
    let mut output: Box<dyn io::Write>;

    let format = if args.short {
        changes::Format::Short
    } else {
        changes::Format::Long
    };

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
        if changes::show(format, &taskdb, change, &mut output).is_err() {
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
