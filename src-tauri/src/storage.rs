use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, Connection, OptionalExtension};

use crate::model::{AppSettings, AttemptReservation, ControlIntent, QueueTask};

const SCHEMA_VERSION: i64 = 5;

pub struct ActionOutcome {
    pub task: QueueTask,
    pub control: Option<ControlIntent>,
    pub should_dispatch: bool,
}

pub fn unix_timestamp() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|_| "clockError".to_string())
}

pub fn open_database(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path).map_err(|_| "storageError".to_string())?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(|_| "storageError".to_string())?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|_| "storageError".to_string())?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|_| "storageError".to_string())?;
    migrate(&connection)?;
    Ok(connection)
}

pub fn recover_interrupted(connection: &Connection) -> Result<usize, String> {
    let timestamp = unix_timestamp()?;
    let changed = connection
        .execute(
            "UPDATE tasks
             SET state = 'interrupted', error_code = 'interruptedAfterRestart',
                 revision = revision + 1, updated_at = ?1
             WHERE state IN ('starting', 'downloading', 'postprocessing', 'pausing', 'stopping')",
            [timestamp],
        )
        .map_err(|_| "storageError".to_string())?;
    connection
        .execute(
            "UPDATE attempts
             SET finished_at = ?1, result = 'interrupted'
             WHERE finished_at IS NULL",
            [timestamp],
        )
        .map_err(|_| "storageError".to_string())?;
    Ok(changed)
}

pub fn prepare_shutdown(
    connection: &mut Connection,
) -> Result<Vec<(String, ControlIntent)>, String> {
    let timestamp = unix_timestamp()?;
    let transaction = connection
        .transaction()
        .map_err(|_| "storageError".to_string())?;
    let mut statement = transaction
        .prepare(
            "SELECT id, state FROM tasks
             WHERE state IN ('starting', 'downloading', 'postprocessing', 'pausing', 'stopping')",
        )
        .map_err(|_| "storageError".to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|_| "storageError".to_string())?;
    let controls = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "storageError".to_string())?
        .into_iter()
        .map(|(id, state)| {
            let intent = if state == "stopping" {
                ControlIntent::Stop
            } else {
                ControlIntent::Pause
            };
            (id, intent)
        })
        .collect::<Vec<_>>();
    drop(statement);

    transaction
        .execute(
            "UPDATE tasks
             SET state = 'pausing', revision = revision + 1, updated_at = ?1
             WHERE state IN ('starting', 'downloading', 'postprocessing')",
            [timestamp],
        )
        .map_err(|_| "storageError".to_string())?;
    transaction
        .commit()
        .map_err(|_| "storageError".to_string())?;
    Ok(controls)
}

pub fn settle_orphaned_control(
    connection: &mut Connection,
    task_id: &str,
    intent: ControlIntent,
) -> Result<QueueTask, String> {
    let timestamp = unix_timestamp()?;
    let transaction = connection
        .transaction()
        .map_err(|_| "storageError".to_string())?;
    let changed = transaction
        .execute(
            "UPDATE tasks
             SET state = ?1, speed = NULL, eta = NULL, error_code = NULL,
                 revision = revision + 1, updated_at = ?2
             WHERE id = ?3 AND state = ?4",
            params![
                intent.stable_state(),
                timestamp,
                task_id,
                intent.transient_state()
            ],
        )
        .map_err(|_| "storageError".to_string())?;
    if changed != 1 {
        return Err("revisionConflict".to_string());
    }
    transaction
        .execute(
            "UPDATE attempts SET finished_at = ?1, result = ?2, error_code = NULL
             WHERE task_id = ?3 AND finished_at IS NULL",
            params![timestamp, intent.stable_state(), task_id],
        )
        .map_err(|_| "storageError".to_string())?;
    let task = load_task(&transaction, task_id)?;
    transaction
        .commit()
        .map_err(|_| "storageError".to_string())?;
    Ok(task)
}

pub fn list_tasks(connection: &Connection) -> Result<Vec<QueueTask>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, url, state, revision, created_at, updated_at, progress,
                    downloaded_bytes, total_bytes, speed, eta, output_path,
                    error_code, attempt_count
             FROM tasks ORDER BY created_at DESC, id DESC",
        )
        .map_err(|_| "storageError".to_string())?;
    let rows = statement
        .query_map([], row_to_task)
        .map_err(|_| "storageError".to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| "storageError".to_string())
}

pub fn load_settings(
    connection: &Connection,
    default_output_directory: &Path,
) -> Result<AppSettings, String> {
    let default_output = default_output_directory.to_string_lossy().into_owned();
    connection
        .execute(
            "INSERT OR IGNORE INTO app_settings
             (id, concurrency, speed_limit_bytes_per_second, browser_for_cookies, output_directory)
             VALUES (1, 2, NULL, NULL, ?1)",
            [&default_output],
        )
        .map_err(|_| "storageError".to_string())?;
    connection
        .query_row(
            "SELECT concurrency, speed_limit_bytes_per_second, browser_for_cookies, output_directory
             FROM app_settings WHERE id = 1",
            [],
            |row| {
                let concurrency = row.get::<_, i64>(0)?;
                let speed_limit = row.get::<_, Option<i64>>(1)?;
                Ok(AppSettings {
                    concurrency: usize::try_from(concurrency).unwrap_or_default(),
                    speed_limit_bytes_per_second: speed_limit
                        .and_then(|value| u64::try_from(value).ok()),
                    browser_for_cookies: row.get(2)?,
                    output_directory: row.get(3)?,
                })
            },
        )
        .map_err(|_| "storageError".to_string())
}

pub fn save_settings(connection: &mut Connection, settings: &AppSettings) -> Result<(), String> {
    let concurrency = i64::try_from(settings.concurrency).map_err(|_| "invalidConcurrency")?;
    let speed_limit = settings
        .speed_limit_bytes_per_second
        .map(i64::try_from)
        .transpose()
        .map_err(|_| "invalidSpeedLimit")?;
    let transaction = connection
        .transaction()
        .map_err(|_| "storageError".to_string())?;
    transaction
        .execute(
            "INSERT INTO app_settings
             (id, concurrency, speed_limit_bytes_per_second, browser_for_cookies, output_directory)
             VALUES (1, ?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
               concurrency = excluded.concurrency,
               speed_limit_bytes_per_second = excluded.speed_limit_bytes_per_second,
               browser_for_cookies = excluded.browser_for_cookies,
               output_directory = excluded.output_directory",
            params![
                concurrency,
                speed_limit,
                settings.browser_for_cookies,
                settings.output_directory
            ],
        )
        .map_err(|_| "storageError".to_string())?;
    transaction.commit().map_err(|_| "storageError".to_string())
}

pub fn load_task(connection: &Connection, task_id: &str) -> Result<QueueTask, String> {
    connection
        .query_row(
            "SELECT id, url, state, revision, created_at, updated_at, progress,
                    downloaded_bytes, total_bytes, speed, eta, output_path,
                    error_code, attempt_count
             FROM tasks WHERE id = ?1",
            [task_id],
            row_to_task,
        )
        .optional()
        .map_err(|_| "storageError".to_string())?
        .ok_or_else(|| "taskNotFound".to_string())
}

pub fn insert_task(connection: &Connection, url: &str) -> Result<QueueTask, String> {
    insert_task_with_format(connection, url, None)
}

pub fn insert_task_with_format(
    connection: &Connection,
    url: &str,
    format_selector: Option<&str>,
) -> Result<QueueTask, String> {
    let timestamp = unix_timestamp()?;
    let id = random_id(connection)?;
    connection
        .execute(
            "INSERT INTO tasks
             (id, url, state, revision, created_at, updated_at, format_selector)
             VALUES (?1, ?2, 'queued', 0, ?3, ?3, ?4)",
            params![id, url, timestamp, format_selector],
        )
        .map_err(|_| "storageError".to_string())?;
    load_task(connection, &id)
}

pub fn apply_action(
    connection: &mut Connection,
    task_id: &str,
    action: &str,
    expected_revision: i64,
) -> Result<ActionOutcome, String> {
    let timestamp = unix_timestamp()?;
    let transaction = connection
        .transaction()
        .map_err(|_| "storageError".to_string())?;
    let current = load_task(&transaction, task_id)?;
    if current.revision != expected_revision {
        return Err("revisionConflict".to_string());
    }

    let (next_state, control, should_dispatch) = transition(&current.state, action)?;
    let changed = transaction
        .execute(
            "UPDATE tasks
             SET state = ?1, revision = revision + 1, updated_at = ?2,
                 error_code = CASE WHEN ?1 = 'queued' THEN NULL ELSE error_code END
             WHERE id = ?3 AND revision = ?4",
            params![next_state, timestamp, task_id, expected_revision],
        )
        .map_err(|_| "storageError".to_string())?;
    if changed != 1 {
        return Err("revisionConflict".to_string());
    }
    let task = load_task(&transaction, task_id)?;
    transaction
        .commit()
        .map_err(|_| "storageError".to_string())?;
    Ok(ActionOutcome {
        task,
        control,
        should_dispatch,
    })
}

pub fn remove_task(
    connection: &Connection,
    task_id: &str,
    expected_revision: i64,
) -> Result<(), String> {
    let current = load_task(connection, task_id)?;
    if current.revision != expected_revision {
        return Err("revisionConflict".to_string());
    }
    if !matches!(
        current.state.as_str(),
        "paused" | "stopped" | "completed" | "failed" | "interrupted"
    ) {
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

pub fn reserve_next_attempt(
    connection: &mut Connection,
) -> Result<Option<AttemptReservation>, String> {
    let timestamp = unix_timestamp()?;
    let transaction = connection
        .transaction()
        .map_err(|_| "storageError".to_string())?;
    let candidate = transaction
        .query_row(
            "SELECT id, url, format_selector FROM tasks WHERE state = 'queued'
             ORDER BY created_at ASC, id ASC LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|_| "storageError".to_string())?;
    let Some((task_id, url, format_selector)) = candidate else {
        return Ok(None);
    };
    let attempt_id = random_id(&transaction)?;
    let changed = transaction
        .execute(
            "UPDATE tasks SET state = 'starting', revision = revision + 1,
                    updated_at = ?1, progress = 0, downloaded_bytes = 0,
                    total_bytes = NULL, speed = NULL, eta = NULL,
                    output_path = NULL, error_code = NULL,
                    attempt_count = attempt_count + 1
             WHERE id = ?2 AND state = 'queued'",
            params![timestamp, task_id],
        )
        .map_err(|_| "storageError".to_string())?;
    if changed != 1 {
        return Ok(None);
    }
    transaction
        .execute(
            "INSERT INTO attempts (id, task_id, started_at) VALUES (?1, ?2, ?3)",
            params![attempt_id, task_id, timestamp],
        )
        .map_err(|_| "storageError".to_string())?;
    transaction
        .commit()
        .map_err(|_| "storageError".to_string())?;
    Ok(Some(AttemptReservation {
        task_id,
        attempt_id,
        url,
        format_selector,
    }))
}

pub fn mark_started(
    connection: &Connection,
    reservation: &AttemptReservation,
    pid: u32,
    engine_version: &str,
) -> Result<(), String> {
    let timestamp = unix_timestamp()?;
    connection
        .execute(
            "UPDATE tasks SET state = 'downloading', revision = revision + 1, updated_at = ?1
             WHERE id = ?2 AND state = 'starting'",
            params![timestamp, reservation.task_id],
        )
        .map_err(|_| "storageError".to_string())?;
    connection
        .execute(
            "UPDATE attempts SET pid = ?1, engine_version = ?2 WHERE id = ?3",
            params![i64::from(pid), engine_version, reservation.attempt_id],
        )
        .map_err(|_| "storageError".to_string())?;
    Ok(())
}

pub fn update_progress(
    connection: &Connection,
    task_id: &str,
    downloaded_bytes: i64,
    total_bytes: Option<i64>,
    speed: Option<f64>,
    eta: Option<i64>,
) -> Result<(), String> {
    let progress = total_bytes
        .filter(|total| *total > 0)
        .map(|total| (downloaded_bytes as f64 / total as f64).clamp(0.0, 1.0))
        .unwrap_or(0.0);
    connection
        .execute(
            "UPDATE tasks SET progress = ?1, downloaded_bytes = ?2, total_bytes = ?3,
                    speed = ?4, eta = ?5
             WHERE id = ?6 AND state = 'downloading'",
            params![progress, downloaded_bytes, total_bytes, speed, eta, task_id],
        )
        .map_err(|_| "storageError".to_string())?;
    Ok(())
}

pub fn mark_postprocessing(connection: &Connection, task_id: &str) -> Result<(), String> {
    let timestamp = unix_timestamp()?;
    connection
        .execute(
            "UPDATE tasks SET state = 'postprocessing', progress = 1.0,
                    revision = revision + 1, updated_at = ?1
             WHERE id = ?2 AND state = 'downloading'",
            params![timestamp, task_id],
        )
        .map_err(|_| "storageError".to_string())?;
    Ok(())
}

pub fn record_output(
    connection: &Connection,
    task_id: &str,
    path: &str,
    size: i64,
) -> Result<(), String> {
    if size <= 0 {
        return Err("outputMissing".to_string());
    }
    connection
        .execute(
            "UPDATE tasks SET output_path = ?1, downloaded_bytes = ?2,
                    total_bytes = ?2, progress = 1.0
             WHERE id = ?3",
            params![path, size, task_id],
        )
        .map_err(|_| "storageError".to_string())?;
    Ok(())
}

pub fn finalize_attempt(
    connection: &mut Connection,
    reservation: &AttemptReservation,
    control: Option<ControlIntent>,
    success: bool,
    error_code: Option<&str>,
) -> Result<(), String> {
    let timestamp = unix_timestamp()?;
    let transaction = connection
        .transaction()
        .map_err(|_| "storageError".to_string())?;
    let current = load_task(&transaction, &reservation.task_id)?;
    let state = control
        .map(ControlIntent::stable_state)
        .unwrap_or(if success { "completed" } else { "failed" });
    let resolved_error = if success || control.is_some() {
        None
    } else {
        error_code
    };
    transaction
        .execute(
            "UPDATE tasks SET state = ?1, progress = CASE WHEN ?1 = 'completed' THEN 1.0 ELSE progress END,
                    speed = NULL, eta = NULL, error_code = ?2,
                    revision = revision + 1, updated_at = ?3
             WHERE id = ?4 AND revision = ?5",
            params![state, resolved_error, timestamp, reservation.task_id, current.revision],
        )
        .map_err(|_| "storageError".to_string())?;
    transaction
        .execute(
            "UPDATE attempts SET finished_at = ?1, result = ?2, error_code = ?3
             WHERE id = ?4",
            params![timestamp, state, resolved_error, reservation.attempt_id],
        )
        .map_err(|_| "storageError".to_string())?;
    transaction.commit().map_err(|_| "storageError".to_string())
}

pub fn fail_reservation(
    connection: &mut Connection,
    reservation: &AttemptReservation,
    error_code: &str,
) -> Result<(), String> {
    finalize_attempt(connection, reservation, None, false, Some(error_code))
}

fn transition(
    current: &str,
    action: &str,
) -> Result<(&'static str, Option<ControlIntent>, bool), String> {
    match (current, action) {
        ("queued", "pause") => Ok(("paused", None, false)),
        ("starting" | "downloading" | "postprocessing", "pause") => Ok((
            ControlIntent::Pause.transient_state(),
            Some(ControlIntent::Pause),
            false,
        )),
        ("paused" | "stopped" | "interrupted" | "failed", "resume") => Ok(("queued", None, true)),
        ("queued" | "paused", "stop") => Ok(("stopped", None, false)),
        ("starting" | "downloading" | "postprocessing" | "pausing", "stop") => Ok((
            ControlIntent::Stop.transient_state(),
            Some(ControlIntent::Stop),
            false,
        )),
        _ => Err("invalidAction".to_string()),
    }
}

fn random_id(connection: &Connection) -> Result<String, String> {
    connection
        .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
        .map_err(|_| "storageError".to_string())
}

fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<QueueTask> {
    Ok(QueueTask {
        id: row.get(0)?,
        url: row.get(1)?,
        state: row.get(2)?,
        revision: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        progress: row.get(6)?,
        downloaded_bytes: row.get(7)?,
        total_bytes: row.get(8)?,
        speed: row.get(9)?,
        eta: row.get(10)?,
        output_path: row.get(11)?,
        error_code: row.get(12)?,
        attempt_count: row.get(13)?,
    })
}

fn migrate(connection: &Connection) -> Result<(), String> {
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| "storageError".to_string())?;
    if version > SCHEMA_VERSION {
        return Err("databaseTooNew".to_string());
    }

    let has_tasks: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'tasks')",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "storageError".to_string())?;

    if !has_tasks {
        connection
            .execute_batch(&format!(
                "{} PRAGMA user_version = {SCHEMA_VERSION};",
                schema_sql()
            ))
            .map_err(|_| "storageError".to_string())?;
    } else if version < 2 {
        migrate_legacy_tasks(connection)?;
    } else {
        if version < 3 {
            connection
                .execute_batch(settings_schema_v3_sql())
                .map_err(|_| "storageError".to_string())?;
        }
        if version < 4 {
            migrate_browser_cookie_setting(connection)?;
        }
        if version < 5 {
            migrate_task_format_selection(connection)?;
        }
        connection
            .execute_batch(schema_sql())
            .map_err(|_| "storageError".to_string())?;
        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|_| "storageError".to_string())?;
    }
    Ok(())
}

fn migrate_browser_cookie_setting(connection: &Connection) -> Result<(), String> {
    let has_column: bool = connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM pragma_table_info('app_settings')
                WHERE name = 'browser_for_cookies'
            )",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "storageError".to_string())?;
    if !has_column {
        connection
            .execute(
                "ALTER TABLE app_settings ADD COLUMN browser_for_cookies TEXT",
                [],
            )
            .map_err(|_| "storageError".to_string())?;
    }
    Ok(())
}

fn migrate_task_format_selection(connection: &Connection) -> Result<(), String> {
    let has_column: bool = connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM pragma_table_info('tasks')
                WHERE name = 'format_selector'
            )",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "storageError".to_string())?;
    if !has_column {
        connection
            .execute("ALTER TABLE tasks ADD COLUMN format_selector TEXT", [])
            .map_err(|_| "storageError".to_string())?;
    }
    Ok(())
}

fn migrate_legacy_tasks(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(&format!(
            "BEGIN IMMEDIATE;
             DROP INDEX IF EXISTS idx_tasks_created_at;
             DROP INDEX IF EXISTS idx_tasks_dispatch;
             ALTER TABLE tasks RENAME TO tasks_legacy;
             {}
             INSERT INTO tasks (id, url, state, revision, created_at, updated_at)
             SELECT id, url,
                    CASE WHEN state = 'running' THEN 'interrupted' ELSE state END,
                    revision, created_at, updated_at
             FROM tasks_legacy;
             DROP TABLE tasks_legacy;
             PRAGMA user_version = {SCHEMA_VERSION};
             COMMIT;",
            schema_sql()
        ))
        .map_err(|_| "storageError".to_string())
}

fn schema_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS tasks (
        id TEXT PRIMARY KEY,
        url TEXT NOT NULL,
        state TEXT NOT NULL CHECK (state IN (
            'queued', 'starting', 'downloading', 'postprocessing', 'pausing',
            'paused', 'stopping', 'stopped', 'completed', 'failed', 'interrupted'
        )),
        revision INTEGER NOT NULL DEFAULT 0,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        progress REAL NOT NULL DEFAULT 0,
        downloaded_bytes INTEGER NOT NULL DEFAULT 0,
        total_bytes INTEGER,
        speed REAL,
        eta INTEGER,
        output_path TEXT,
        error_code TEXT,
        attempt_count INTEGER NOT NULL DEFAULT 0,
        format_selector TEXT
    );
    CREATE INDEX IF NOT EXISTS idx_tasks_dispatch
        ON tasks(state, created_at ASC, id ASC);
    CREATE INDEX IF NOT EXISTS idx_tasks_created_at
        ON tasks(created_at DESC, id DESC);
    CREATE TABLE IF NOT EXISTS attempts (
        id TEXT PRIMARY KEY,
        task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
        started_at INTEGER NOT NULL,
        finished_at INTEGER,
        pid INTEGER,
        engine_version TEXT,
        result TEXT,
        error_code TEXT
    );
    CREATE INDEX IF NOT EXISTS idx_attempts_task
        ON attempts(task_id, started_at DESC);
    CREATE TABLE IF NOT EXISTS app_settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        concurrency INTEGER NOT NULL,
        speed_limit_bytes_per_second INTEGER,
        browser_for_cookies TEXT,
        output_directory TEXT NOT NULL
    );"
}

fn settings_schema_v3_sql() -> &'static str {
    "CREATE TABLE IF NOT EXISTS app_settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        concurrency INTEGER NOT NULL,
        speed_limit_bytes_per_second INTEGER,
        output_directory TEXT NOT NULL
    );"
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{
        apply_action, insert_task, insert_task_with_format, load_settings, open_database,
        prepare_shutdown, recover_interrupted, reserve_next_attempt, save_settings,
        settle_orphaned_control,
    };
    use crate::model::{AppSettings, ControlIntent};

    #[test]
    fn migrates_legacy_queue_without_losing_tasks() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("queue.sqlite3");
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE tasks (
                        id TEXT PRIMARY KEY, url TEXT NOT NULL,
                        state TEXT NOT NULL, revision INTEGER NOT NULL,
                        created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
                    );
                    INSERT INTO tasks VALUES ('one', 'https://example.com/a', 'queued', 0, 1, 1);",
                )
                .unwrap();
        }
        let connection = open_database(&path).unwrap();
        let tasks = super::list_tasks(&connection).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].url, "https://example.com/a");
    }

    #[test]
    fn migrates_v2_and_persists_download_settings() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("queue.sqlite3");
        {
            let connection = open_database(&path).unwrap();
            insert_task(&connection, "https://example.com/a").unwrap();
            connection.execute("DROP TABLE app_settings", []).unwrap();
            connection.pragma_update(None, "user_version", 2).unwrap();
        }

        let mut connection = open_database(&path).unwrap();
        let default_output = directory.path().join("Downloads/JiveFetch");
        let defaults = load_settings(&connection, &default_output).unwrap();
        assert_eq!(defaults.concurrency, 2);
        assert_eq!(defaults.speed_limit_bytes_per_second, None);
        assert_eq!(defaults.browser_for_cookies, None);
        assert_eq!(super::list_tasks(&connection).unwrap().len(), 1);

        let configured = AppSettings {
            concurrency: 7,
            speed_limit_bytes_per_second: Some(2 * 1024 * 1024),
            browser_for_cookies: Some("firefox".to_string()),
            output_directory: directory
                .path()
                .join("Media")
                .to_string_lossy()
                .into_owned(),
        };
        save_settings(&mut connection, &configured).unwrap();
        assert_eq!(
            load_settings(&connection, &default_output).unwrap(),
            configured
        );
    }

    #[test]
    fn migrates_v3_settings_without_cookie_column() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("queue.sqlite3");
        {
            let connection = open_database(&path).unwrap();
            connection
                .execute_batch(
                    "DROP TABLE app_settings;
                     CREATE TABLE app_settings (
                       id INTEGER PRIMARY KEY CHECK (id = 1),
                       concurrency INTEGER NOT NULL,
                       speed_limit_bytes_per_second INTEGER,
                       output_directory TEXT NOT NULL
                     );
                     INSERT INTO app_settings VALUES (1, 4, 524288, '/tmp/JiveFetch');
                     PRAGMA user_version = 3;",
                )
                .unwrap();
        }

        let connection = open_database(&path).unwrap();
        let settings = load_settings(&connection, directory.path()).unwrap();
        assert_eq!(settings.concurrency, 4);
        assert_eq!(settings.speed_limit_bytes_per_second, Some(512 * 1024));
        assert_eq!(settings.browser_for_cookies, None);
        assert_eq!(settings.output_directory, "/tmp/JiveFetch");
    }

    #[test]
    fn migrates_v4_queue_and_reserves_the_selected_format() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("queue.sqlite3");
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE tasks (
                        id TEXT PRIMARY KEY,
                        url TEXT NOT NULL,
                        state TEXT NOT NULL,
                        revision INTEGER NOT NULL DEFAULT 0,
                        created_at INTEGER NOT NULL,
                        updated_at INTEGER NOT NULL,
                        progress REAL NOT NULL DEFAULT 0,
                        downloaded_bytes INTEGER NOT NULL DEFAULT 0,
                        total_bytes INTEGER,
                        speed REAL,
                        eta INTEGER,
                        output_path TEXT,
                        error_code TEXT,
                        attempt_count INTEGER NOT NULL DEFAULT 0
                     );
                     CREATE TABLE app_settings (
                        id INTEGER PRIMARY KEY CHECK (id = 1),
                        concurrency INTEGER NOT NULL,
                        speed_limit_bytes_per_second INTEGER,
                        browser_for_cookies TEXT,
                        output_directory TEXT NOT NULL
                     );
                     PRAGMA user_version = 4;",
                )
                .unwrap();
        }

        let mut connection = open_database(&path).unwrap();
        let task = insert_task_with_format(
            &connection,
            "https://example.com/video",
            Some("137+bestaudio/137/best"),
        )
        .unwrap();
        let reservation = reserve_next_attempt(&mut connection).unwrap().unwrap();
        assert_eq!(reservation.task_id, task.id);
        assert_eq!(
            reservation.format_selector.as_deref(),
            Some("137+bestaudio/137/best")
        );
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 5);
    }

    #[test]
    fn actions_are_revision_checked_and_explicit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("queue.sqlite3");
        let mut connection = open_database(&path).unwrap();
        let task = insert_task(&connection, "https://example.com/a").unwrap();
        let paused = apply_action(&mut connection, &task.id, "pause", task.revision).unwrap();
        assert_eq!(paused.task.state, "paused");
        assert!(apply_action(&mut connection, &task.id, "resume", task.revision).is_err());
        let queued =
            apply_action(&mut connection, &task.id, "resume", paused.task.revision).unwrap();
        assert_eq!(queued.task.state, "queued");
        assert!(queued.should_dispatch);
    }

    #[test]
    fn startup_recovery_marks_transient_tasks_interrupted() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("queue.sqlite3");
        let connection = open_database(&path).unwrap();
        let task = insert_task(&connection, "https://example.com/a").unwrap();
        connection
            .execute(
                "UPDATE tasks SET state = 'downloading' WHERE id = ?1",
                [&task.id],
            )
            .unwrap();
        assert_eq!(recover_interrupted(&connection).unwrap(), 1);
        assert_eq!(
            super::load_task(&connection, &task.id).unwrap().state,
            "interrupted"
        );
    }

    #[test]
    fn shutdown_persists_pause_intent_before_control() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("queue.sqlite3");
        let mut connection = open_database(&path).unwrap();
        let task = insert_task(&connection, "https://example.com/a").unwrap();
        connection
            .execute(
                "UPDATE tasks SET state = 'downloading' WHERE id = ?1",
                [&task.id],
            )
            .unwrap();

        let controls = prepare_shutdown(&mut connection).unwrap();
        assert_eq!(controls, vec![(task.id.clone(), ControlIntent::Pause)]);
        assert_eq!(
            super::load_task(&connection, &task.id).unwrap().state,
            "pausing"
        );

        let settled =
            settle_orphaned_control(&mut connection, &task.id, ControlIntent::Pause).unwrap();
        assert_eq!(settled.state, "paused");
    }
}
