//! Print changes from the Taskwarrior database.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io;

use crate::taskdb::{Change, TaskDb};

use chrono::{Local, TimeZone};
use colored::Colorize;

pub fn show(task_db: &TaskDb, change: &Change, mut output: impl io::Write) -> io::Result<()> {
    let local_time = Local.timestamp(change.time, 0);

    // Header
    write!(output, "{}", local_time.format("%F %X"))?;

    if let Some(uuid) = change.new.get("uuid") {
        if let Some(task) = task_db.tasks.get(uuid) {
            match task.id {
                Some(id) if id != 0 => write!(output, " [{}]", id)?,
                _ => write!(output, " [{}]", uuid.split('-').next().unwrap())?,
            }

            if let Some(description) = &task.description {
                write!(output, " {}", description.bold())?;
            }
        }
    }

    writeln!(output)?;

    // Fields
    match &change.old {
        None => new_fields(&mut output, &change.new)?,
        Some(old) => diff(&mut output, &change.new, old)?,
    }

    writeln!(output)
}

pub fn new_fields(output: &mut impl io::Write, fields: &HashMap<String, String>) -> io::Result<()> {
    for (key, value) in fields {
        writeln!(output, "  {}: {}", key, format_value(value).green())?;
    }

    Ok(())
}

pub fn diff(
    output: &mut impl io::Write,
    new: &HashMap<String, String>,
    old: &HashMap<String, String>,
) -> io::Result<()> {
    let all_fields = {
        let mut set = HashSet::new();
        for map in &[old, new] {
            for k in map.keys() {
                set.insert(k);
            }
        }
        set
    };

    for field in all_fields {
        let old_value = old.get(field);
        let new_value = new.get(field);

        if old_value == new_value {
            continue;
        }

        writeln!(output, "  {}:", field)?;

        if let Some(value) = old_value {
            writeln!(output, "    - {}", format_value(value).red())?;
        }

        if let Some(value) = new_value {
            writeln!(output, "    + {}", format_value(value).green())?;
        }
    }

    Ok(())
}

/// Format a value read from the undo database.
///
/// If the value looks like a timestamp, it returns a formatted date.
/// If not, it returns the original value.
fn format_value(value: &str) -> Cow<str> {
    lazy_static::lazy_static! {
        static ref NOW: i64 = chrono::Utc::now().timestamp();
    }

    const YEAR: i64 = 365 * 24 * 60 * 60;
    let time_range = *NOW - YEAR..=*NOW + YEAR;

    // Try to convert the value to a timestamp.
    if let Ok(ts) = value.parse() {
        if time_range.contains(&ts) {
            let localtime = chrono::Local.timestamp(ts, 0);
            let delta = delta_time(*NOW - ts);
            return format!("{}{}", localtime.format("%F %X %Z"), delta).into();
        }
    }

    value.into()
}

/// Format a string to represent the time distance.
fn delta_time(delta: i64) -> String {
    let delta_abs = delta.abs();

    if delta_abs > 90 * 24 * 60 * 60 {
        // Ignore +90 days
        return String::new();
    };

    let value;
    let unit;

    if delta_abs > 3 * 24 * 60 * 60 {
        value = delta_abs / (24 * 60 * 60);
        unit = "days";
    } else if delta_abs > 2 * 60 * 60 {
        value = delta_abs / (60 * 60);
        unit = "hours";
    } else if delta_abs > 2 * 60 {
        value = delta_abs / 60;
        unit = "minutes";
    } else {
        value = delta_abs;
        unit = "seconds";
    }

    let suffix = if delta > 0 { "ago" } else { "from now" };
    format!(" ({} {} {})", value, unit, suffix)
}
