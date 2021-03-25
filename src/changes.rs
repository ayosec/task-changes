//! Print changes from the Taskwarrior database.

use std::collections::{HashMap, HashSet};
use std::io;

use crate::taskdb::{Change, TaskDb};
use chrono::{Local, TimeZone};

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
                write!(output, " {}", description)?;
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
        writeln!(output, "  {}: {}", key, value)?;
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
            writeln!(output, "    - {}", value)?;
        }

        if let Some(value) = new_value {
            writeln!(output, "    + {}", value)?;
        }
    }

    Ok(())
}
