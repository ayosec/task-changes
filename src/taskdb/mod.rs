//! Wrapper to access Taskwarrior database.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;

mod undodata;

pub use undodata::Change;

pub struct TaskDb {
    pub changes: Vec<Change>,
    pub tasks: HashMap<String, Task>,
}

impl TaskDb {
    pub fn new(command_path: &OsStr) -> anyhow::Result<TaskDb> {
        // Get path where undo.data file.
        let data_location = match get_data_location(command_path)? {
            Some(dl) => dl,
            None => {
                anyhow::bail!("Missing data.location")
            }
        };

        let tasks = get_tasks(command_path)?;

        let changes = undodata::parse(data_location.join("undo.data"))?;

        Ok(TaskDb { changes, tasks })
    }
}

/// Execute the task command and capture its output.
fn run_cli(path: &OsStr, arg: &str) -> anyhow::Result<Vec<u8>> {
    let stdout = Command::new(path)
        .arg(arg)
        .output()
        .with_context(|| format!("Failed to execute {:?}", path))?
        .stdout;

    Ok(stdout)
}

/// Extract from Taskwarrior configuration the value of the data.location item.
#[allow(clippy::never_loop)]
fn get_data_location(path: &OsStr) -> anyhow::Result<Option<PathBuf>> {
    let output = run_cli(path, "_show")?;
    let mut last = 0;

    let data_location = 'result: loop {
        for index in memchr::memchr_iter(b'\n', &output) {
            if let Some(value) = output[last..index].strip_prefix(b"data.location=") {
                break 'result value;
            }

            last = index + 1;
        }

        return Ok(None);
    };

    let path = match data_location.strip_prefix(b"~/") {
        Some(tail) => {
            let tail = OsStr::from_bytes(tail);
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(tail))
        }

        None => Some(PathBuf::from(OsStr::from_bytes(data_location))),
    };

    Ok(path)
}

/// Task data read from the export command.
#[derive(serde::Deserialize)]
pub struct Task {
    pub id: Option<isize>,
    pub uuid: Option<String>,
    pub description: Option<String>,
    pub project: Option<String>,
    pub status: Option<String>,
}

pub fn get_tasks(path: &OsStr) -> anyhow::Result<HashMap<String, Task>> {
    let output = run_cli(path, "export")?;
    let tasks: Vec<Task> = serde_json::from_slice(&output)?;

    let mut map = HashMap::with_capacity(tasks.len());
    for task in tasks {
        if let Some(uuid) = &task.uuid {
            map.insert(uuid.clone(), task);
        }
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::TaskDb;
    use std::ffi::OsStr;

    #[test]
    fn load_task_data() {
        let task_db = TaskDb::new(OsStr::new("src/taskdb/tests/task_mock.sh")).unwrap();

        // Data from `task export`.

        assert_eq!(task_db.tasks.len(), 2);
        assert_eq!(
            task_db
                .tasks
                .get("e505f4ba-cb73-42a7-9301-a4b2c68533c9")
                .unwrap()
                .id,
            Some(2)
        );

        // Changes in `undo.data` file.

        assert_eq!(task_db.changes.len(), 5);
        assert_eq!(task_db.changes[0].time, 1616626518);

        let change = &task_db.changes[4];
        assert_eq!(change.time, 1616626594);
        assert_eq!(change.new["annotation_1616626594"].as_str(), "a\nb");

        let old = change.old.as_ref().unwrap();
        assert_eq!(
            old["annotation_1616626556"],
            r#"http://example.com: "data""#
        );
        assert_eq!(old["tags"], "next");
    }
}
