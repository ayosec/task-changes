//! Run an external program as a pager (like "less")

use std::env;
use std::process::{Command, Stdio};

pub const PAGER_ENV: &str = "TASKWARRIOR_PAGER";

const DEFAULT_PAGER: &str = if cfg!(windows) { "more" } else { "pager -F -R" };

pub fn command() -> Option<Command> {
    if !atty::is(atty::Stream::Stdout) {
        return None;
    }

    let pager_var = env::var(PAGER_ENV);
    let pager_args = pager_var.as_deref().unwrap_or(DEFAULT_PAGER);

    let mut pager_args = match shell_words::split(pager_args) {
        Ok(words) => words.into_iter(),
        Err(e) => {
            eprintln!("Failed to parse {}: {:?}", PAGER_ENV, e);
            return None;
        }
    };

    let mut cmd = Command::new(pager_args.next()?);

    for arg in pager_args {
        cmd.arg(arg);
    }

    cmd.stdin(Stdio::piped());

    Some(cmd)
}
