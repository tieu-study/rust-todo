use std::fmt;
use std::fs::{OpenOptions, File};
use std::io::{ErrorKind, Result, Seek, SeekFrom, Error};  // Include the `Error` type.
use chrono::{serde::ts_seconds, DateTime, Local, Utc};
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub text: String,

    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Task {
    pub fn new(text: String) -> Task {
        let created_at: DateTime<Utc> = Utc::now();
        Task { text, created_at }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let created_at = self.created_at.with_timezone(&Local).format("%F %H:%M");
        write!(f, "{:<50} [{}]", self.text, created_at)
    }
}

pub fn add_task(journal_file: PathBuf, task: Task) -> Result<()> {
    // Open the file.
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(journal_file)?;

    // Consume the file's contents as a vector of tasks.
    let mut tasks: Vec<Task> = collect_tasks(&file)?;

    // Add the new task to the vector.
    tasks.push(task);
    serde_json::to_writer(file, &tasks)?;

    Ok(())
}

pub fn complete_task(journal_path: PathBuf, task_position: usize) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(journal_path)?;

    let mut tasks = collect_tasks(&file)?;

    if task_position == 0 || task_position > tasks.len() {
        return Err(Error::new(ErrorKind::InvalidInput, "Invalid task ID"));
    };

    tasks.remove(task_position - 1);

    // truncate the file to zero length
    file.set_len(0)?;

    // write the new tasks to the file
    serde_json::to_writer(file, &tasks)?;
    Ok(())
}

pub fn list_task(journal_path: PathBuf) -> Result<()> {
    // open the file
    let file = OpenOptions::new()
        .read(true)
        .open(journal_path)?;

    let tasks = collect_tasks(&file)?;
    if tasks.is_empty() {
        println!("No tasks found.");
    } else {
        let mut order: i32 = 1;
        for task in tasks {
            println!("{}: {}", order, task);
            order += 1;
        }
    }

    Ok(())
}

fn collect_tasks(mut file: &File) -> Result<Vec<Task>> {
    file.seek(SeekFrom::Start(0))?; // Rewind the file before.
    let tasks = match serde_json::from_reader(file) {
        Ok(tasks) => tasks,
        Err(e) if e.is_eof() => Vec::new(),
        Err(e) => Err(e)?,
    };
    file.seek(SeekFrom::Start(0))?; // Rewind the file after.
    Ok(tasks)
}
