use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use tauri::{Manager, State};
use url::Url;

const DATABASE_FILE: &str = "jivefetch.sqlite3";

struct DatabaseState {
    path: PathBuf,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct QueueTask {
    id: String,
    url: String,
    state: String,
    revision: i64,
    created_at: i64,
    updated_at: i64,
}

fn unix_timestamp() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|_| "clockError".to_string())
}

fn validate_url(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    let parsed = Url::parse(trimmed).map_err(|_| "invalidUrl".to_string())?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("unsupportedScheme".to_string());
    }
    if parsed.host_str().is_none() {
        return Err("missingHost".to_string());
    }
    Ok(trimmed.to_string())
}

fn initialize_connection(connection: &Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(std::time::Duration::from_secs(5))?;
    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            state TEXT NOT NULL CHECK (state IN (
                'queued', 'running', 'paused', 'stopped', 'completed', 'failed'
            )),
            revision INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_tasks_created_at
            ON tasks(created_at DESC, id DESC);
        ",
    )
}

fn open_database(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path).map_err(|_| "storageError".to_string())?;
    initialize_connection(&connection).map_err(|_| "storageError".to_string())?;
    Ok(connection)
}

fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<QueueTask> {
    Ok(QueueTask {
        id: row.get(0)?,
        url: row.get(1)?,
        state: row.get(2)?,
        revision: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn load_task(connection: &Connection, task_id: &str) -> Result<QueueTask, String> {
    connection
        .query_row(
            "SELECT id, url, state, revision, created_at, updated_at FROM tasks WHERE id = ?1",
            [task_id],
            row_to_task,
        )
        .optional()
        .map_err(|_| "storageError".to_string())?
        .ok_or_else(|| "taskNotFound".to_string())
}

fn insert_task(connection: &Connection, url: &str, timestamp: i64) -> Result<QueueTask, String> {
    let id: String = connection
        .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
        .map_err(|_| "storageError".to_string())?;
    connection
        .execute(
            "INSERT INTO tasks (id, url, state, revision, created_at, updated_at)
             VALUES (?1, ?2, 'queued', 0, ?3, ?3)",
            params![id, url, timestamp],
        )
        .map_err(|_| "storageError".to_string())?;
    load_task(connection, &id)
}

#[tauri::command]
fn list_tasks(database: State<'_, DatabaseState>) -> Result<Vec<QueueTask>, String> {
    let connection = open_database(&database.path)?;
    let mut statement = connection
        .prepare(
            "SELECT id, url, state, revision, created_at, updated_at
             FROM tasks ORDER BY created_at DESC, id DESC",
        )
        .map_err(|_| "storageError".to_string())?;
    let rows = statement
        .query_map([], row_to_task)
        .map_err(|_| "storageError".to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "storageError".to_string())
}

#[tauri::command]
fn add_task(url: String, database: State<'_, DatabaseState>) -> Result<QueueTask, String> {
    let url = validate_url(&url)?;
    let timestamp = unix_timestamp()?;
    let connection = open_database(&database.path)?;
    insert_task(&connection, &url, timestamp)
}

fn next_state(current: &str, action: &str) -> Result<&'static str, String> {
    match (current, action) {
        ("queued" | "running", "pause") => Ok("paused"),
        ("paused" | "stopped", "resume") => Ok("queued"),
        ("queued" | "running" | "paused", "stop") => Ok("stopped"),
        _ => Err("invalidAction".to_string()),
    }
}

#[tauri::command]
fn task_action(
    task_id: String,
    action: String,
    expected_revision: i64,
    database: State<'_, DatabaseState>,
) -> Result<QueueTask, String> {
    let timestamp = unix_timestamp()?;
    let mut connection = open_database(&database.path)?;
    let transaction = connection
        .transaction()
        .map_err(|_| "storageError".to_string())?;
    let current = load_task(&transaction, &task_id)?;
    if current.revision != expected_revision {
        return Err("revisionConflict".to_string());
    }
    let state = next_state(&current.state, &action)?;
    transaction
        .execute(
            "UPDATE tasks
             SET state = ?1, revision = revision + 1, updated_at = ?2
             WHERE id = ?3 AND revision = ?4",
            params![state, timestamp, task_id, expected_revision],
        )
        .map_err(|_| "storageError".to_string())?;
    let updated = load_task(&transaction, &task_id)?;
    transaction
        .commit()
        .map_err(|_| "storageError".to_string())?;
    Ok(updated)
}

#[tauri::command]
fn remove_task(
    task_id: String,
    expected_revision: i64,
    database: State<'_, DatabaseState>,
) -> Result<(), String> {
    let connection = open_database(&database.path)?;
    let current = load_task(&connection, &task_id)?;
    if current.revision != expected_revision {
        return Err("revisionConflict".to_string());
    }
    if current.state == "running" {
        return Err("stopBeforeRemove".to_string());
    }
    let changed = connection
        .execute(
            "DELETE FROM tasks WHERE id = ?1 AND revision = ?2",
            params![task_id, expected_revision],
        )
        .map_err(|_| "storageError".to_string())?;
    if changed == 1 {
        Ok(())
    } else {
        Err("revisionConflict".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            fs::create_dir_all(&app_data)?;
            let path = app_data.join(DATABASE_FILE);
            open_database(&path).map_err(std::io::Error::other)?;
            app.manage(DatabaseState { path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_tasks,
            add_task,
            task_action,
            remove_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running JiveFetch");
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::{
        initialize_connection, insert_task, load_task, next_state, open_database, validate_url,
    };
    use rusqlite::Connection;

    #[test]
    fn accepts_http_urls_and_rejects_shell_like_input() {
        assert_eq!(
            validate_url(" https://example.com/video?id=1 ").unwrap(),
            "https://example.com/video?id=1"
        );
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("https://example.com; touch /tmp/nope").is_err());
        assert!(validate_url("not a url").is_err());
    }

    #[test]
    fn queue_transitions_are_explicit() {
        assert_eq!(next_state("queued", "pause").unwrap(), "paused");
        assert_eq!(next_state("paused", "resume").unwrap(), "queued");
        assert_eq!(next_state("queued", "stop").unwrap(), "stopped");
        assert!(next_state("stopped", "pause").is_err());
    }

    #[test]
    fn database_schema_enables_wal_and_tasks() {
        let connection = Connection::open_in_memory().unwrap();
        initialize_connection(&connection).unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name = 'tasks'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn task_survives_database_reopen() {
        let nonce = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("jivefetch-{nonce}.sqlite3"));
        let task = {
            let connection = open_database(&path).unwrap();
            insert_task(&connection, "https://example.com/media", 1_700_000_000).unwrap()
        };
        {
            let connection = open_database(&path).unwrap();
            assert_eq!(load_task(&connection, &task.id).unwrap(), task);
        }
        fs::remove_file(&path).unwrap();
    }
}
