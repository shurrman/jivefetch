use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    engine::{
        parse_engine_line, verified_output_file, EngineEvent, EngineFailureClassifier,
        EngineRegistry,
    },
    model::{AppSettings, AttemptReservation, ControlIntent, EngineStatus, MediaProbe, QueueTask},
    process_supervisor::{ProcessLine, SupervisedProcess},
    storage,
};

const MAX_CONFIGURED_CONCURRENCY: usize = 64;
const MIN_SPEED_LIMIT: u64 = 1024;
const MAX_SPEED_LIMIT: u64 = 10 * 1024 * 1024 * 1024;
const SUPPORTED_COOKIE_BROWSERS: [&str; 9] = [
    "brave", "chrome", "chromium", "edge", "firefox", "opera", "safari", "vivaldi", "whale",
];
const STOP_GRACE: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct SchedulerRuntime {
    inner: Arc<RuntimeInner>,
}

struct RuntimeInner {
    database_path: PathBuf,
    engines: EngineRegistry,
    settings: Mutex<AppSettings>,
    database_writer: Mutex<()>,
    dispatcher: Mutex<()>,
    active: Mutex<HashMap<String, Sender<ControlIntent>>>,
}

impl SchedulerRuntime {
    pub fn new(database_path: PathBuf, output_directory: PathBuf) -> Result<Self, String> {
        let connection = storage::open_database(&database_path)?;
        storage::recover_interrupted(&connection)?;
        let settings = storage::load_settings(&connection, &output_directory)?;
        validate_settings(&settings)?;
        let configured_output = PathBuf::from(&settings.output_directory);
        fs::create_dir_all(&configured_output).map_err(|_| "outputDirectoryError".to_string())?;
        reconcile_completed_output_sizes(&connection, &configured_output)?;
        let engines = EngineRegistry::discover(&engine_probe_directory(&database_path));
        Ok(Self {
            inner: Arc::new(RuntimeInner {
                database_path,
                engines,
                settings: Mutex::new(settings),
                database_writer: Mutex::new(()),
                dispatcher: Mutex::new(()),
                active: Mutex::new(HashMap::new()),
            }),
        })
    }

    pub fn engine_status(&self) -> EngineStatus {
        self.inner.engines.status()
    }

    pub fn settings(&self) -> Result<AppSettings, String> {
        self.inner
            .settings
            .lock()
            .map(|settings| settings.clone())
            .map_err(|_| "schedulerError".to_string())
    }

    pub fn update_settings(&self, mut settings: AppSettings) -> Result<AppSettings, String> {
        settings.output_directory = settings.output_directory.trim().to_string();
        validate_settings(&settings)?;
        fs::create_dir_all(&settings.output_directory)
            .map_err(|_| "outputDirectoryError".to_string())?;
        self.with_database_mut(|connection| storage::save_settings(connection, &settings))?;
        *self
            .inner
            .settings
            .lock()
            .map_err(|_| "schedulerError".to_string())? = settings.clone();
        self.kick();
        Ok(settings)
    }

    pub fn list_tasks(&self) -> Result<Vec<QueueTask>, String> {
        self.with_database(storage::list_tasks)
    }

    pub fn probe_formats(&self, url: &str) -> Result<MediaProbe, String> {
        let settings = self.settings()?;
        self.inner.engines.probe_formats(
            url,
            Path::new(&settings.output_directory),
            settings.browser_for_cookies.as_deref(),
        )
    }

    pub fn add_task(&self, url: &str, format_selector: Option<&str>) -> Result<QueueTask, String> {
        let task = self.with_database(|connection| {
            storage::insert_task_with_format(connection, url, format_selector)
        })?;
        self.kick();
        Ok(task)
    }

    pub fn task_action(
        &self,
        task_id: &str,
        action: &str,
        expected_revision: i64,
    ) -> Result<QueueTask, String> {
        let outcome = self.with_database_mut(|connection| {
            storage::apply_action(connection, task_id, action, expected_revision)
        })?;

        let mut task = outcome.task;
        if let Some(control) = outcome.control {
            let sender = self
                .inner
                .active
                .lock()
                .map_err(|_| "schedulerError".to_string())?
                .get(task_id)
                .cloned();
            if sender.is_none_or(|sender| sender.send(control).is_err()) {
                task = self.with_database_mut(|connection| {
                    storage::settle_orphaned_control(connection, task_id, control)
                })?;
            }
        }
        if outcome.should_dispatch {
            self.kick();
        }
        Ok(task)
    }

    pub fn remove_task(&self, task_id: &str, expected_revision: i64) -> Result<(), String> {
        self.with_database(|connection| {
            storage::remove_task(connection, task_id, expected_revision)
        })
    }

    pub fn kick(&self) {
        let runtime = self.clone();
        thread::spawn(move || runtime.dispatch_available());
    }

    pub fn shutdown(&self) {
        let controls = match self.with_database_mut(storage::prepare_shutdown) {
            Ok(controls) => controls,
            Err(_) => return,
        };
        for (task_id, intent) in controls {
            let sender = self
                .inner
                .active
                .lock()
                .ok()
                .and_then(|active| active.get(&task_id).cloned());
            if sender.is_none_or(|sender| sender.send(intent).is_err()) {
                let _ = self.with_database_mut(|connection| {
                    storage::settle_orphaned_control(connection, &task_id, intent)
                });
            }
        }

        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self
                .inner
                .active
                .lock()
                .map(|active| active.is_empty())
                .unwrap_or(true)
            {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn dispatch_available(&self) {
        let Ok(_dispatch_guard) = self.inner.dispatcher.try_lock() else {
            return;
        };
        loop {
            let active_count = match self.inner.active.lock() {
                Ok(active) => active.len(),
                Err(_) => return,
            };
            let concurrency = match self.settings() {
                Ok(settings) => settings.concurrency,
                Err(_) => return,
            };
            if active_count >= concurrency {
                return;
            }

            let reservation = match self.with_database_mut(storage::reserve_next_attempt) {
                Ok(Some(reservation)) => reservation,
                Ok(None) | Err(_) => return,
            };
            let (control_sender, control_receiver) = mpsc::channel();
            if let Ok(mut active) = self.inner.active.lock() {
                active.insert(reservation.task_id.clone(), control_sender);
            } else {
                let _ = self.with_database_mut(|connection| {
                    storage::fail_reservation(connection, &reservation, "schedulerError")
                });
                return;
            }

            let runtime = self.clone();
            thread::spawn(move || {
                run_attempt(&runtime, &reservation, control_receiver);
                if let Ok(mut active) = runtime.inner.active.lock() {
                    active.remove(&reservation.task_id);
                }
                runtime.kick();
            });
        }
    }

    fn with_database<T>(
        &self,
        operation: impl FnOnce(&rusqlite::Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        let _writer = self
            .inner
            .database_writer
            .lock()
            .map_err(|_| "storageError".to_string())?;
        let connection = storage::open_database(&self.inner.database_path)?;
        operation(&connection)
    }

    fn with_database_mut<T>(
        &self,
        operation: impl FnOnce(&mut rusqlite::Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        let _writer = self
            .inner
            .database_writer
            .lock()
            .map_err(|_| "storageError".to_string())?;
        let mut connection = storage::open_database(&self.inner.database_path)?;
        operation(&mut connection)
    }
}

fn run_attempt(
    runtime: &SchedulerRuntime,
    reservation: &AttemptReservation,
    control_receiver: mpsc::Receiver<ControlIntent>,
) {
    let settings = match runtime.settings() {
        Ok(settings) => settings,
        Err(_) => {
            let _ = runtime.with_database_mut(|connection| {
                storage::fail_reservation(connection, reservation, "schedulerError")
            });
            return;
        }
    };
    let output_directory = PathBuf::from(&settings.output_directory);
    let speed_limit = per_attempt_speed_limit(&settings);
    let plan = match runtime.inner.engines.download_plan(
        &reservation.url,
        &output_directory,
        speed_limit,
        settings.browser_for_cookies.as_deref(),
        reservation.format_selector.as_deref(),
    ) {
        Ok(plan) => plan,
        Err(code) => {
            let _ = runtime.with_database_mut(|connection| {
                storage::fail_reservation(connection, reservation, &code)
            });
            return;
        }
    };

    let mut process =
        match SupervisedProcess::spawn(&plan.executable, &plan.args, &plan.working_directory) {
            Ok(process) => process,
            Err(_) => {
                let _ = runtime.with_database_mut(|connection| {
                    storage::fail_reservation(connection, reservation, "engineSpawnFailed")
                });
                return;
            }
        };

    let _ = runtime.with_database(|connection| {
        storage::mark_started(connection, reservation, process.id(), &plan.engine_version)
    });

    let mut control = None;
    let mut final_output = None;
    let mut exit_success = false;
    let mut process_error = false;
    let mut storage_error = false;
    let mut failure = EngineFailureClassifier::default();

    loop {
        drain_engine_events(
            runtime,
            reservation,
            &process,
            &mut final_output,
            &mut failure,
        );

        if let Ok(intent) = control_receiver.try_recv() {
            control = Some(intent);
            match process.terminate_owned_tree(STOP_GRACE) {
                Ok(_) => {}
                Err(_) => process_error = true,
            }
            break;
        }

        match process.try_wait() {
            Ok(Some(status)) => {
                exit_success = status.success();
                apply_engine_lines(
                    runtime,
                    reservation,
                    process.drain_output_until_closed(Duration::from_millis(250)),
                    &mut final_output,
                    &mut failure,
                );
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(_) => {
                process_error = true;
                break;
            }
        }
    }

    let verified_output = final_output
        .as_deref()
        .and_then(|path| verified_output_file(path, &output_directory));
    if let Some((path, size)) = verified_output.as_ref() {
        storage_error = runtime
            .with_database(|connection| {
                storage::record_output(connection, &reservation.task_id, path, *size)
            })
            .is_err();
    }

    let success = exit_success && verified_output.is_some() && !process_error && !storage_error;
    let error_code = if storage_error {
        Some("storageError")
    } else if process_error {
        Some("processSupervisorError")
    } else if exit_success && verified_output.is_none() {
        Some("outputMissing")
    } else if !exit_success {
        Some(
            failure
                .error_code(settings.browser_for_cookies.is_some())
                .unwrap_or("engineFailed"),
        )
    } else {
        None
    };
    let _ = runtime.with_database_mut(|connection| {
        storage::finalize_attempt(connection, reservation, control, success, error_code)
    });
}

fn reconcile_completed_output_sizes(
    connection: &rusqlite::Connection,
    output_directory: &Path,
) -> Result<usize, String> {
    let mut changed = 0;
    for task in storage::list_tasks(connection)? {
        if task.state != "completed" {
            continue;
        }
        let Some(stored_path) = task.output_path.as_deref() else {
            continue;
        };
        let Some((path, size)) = verified_output_file(Path::new(stored_path), output_directory)
        else {
            continue;
        };
        if task.output_path.as_deref() == Some(path.as_str())
            && task.downloaded_bytes == size
            && task.total_bytes == Some(size)
        {
            continue;
        }
        storage::record_output(connection, &task.id, &path, size)?;
        changed += 1;
    }
    Ok(changed)
}

fn validate_settings(settings: &AppSettings) -> Result<(), String> {
    if !(1..=MAX_CONFIGURED_CONCURRENCY).contains(&settings.concurrency) {
        return Err("invalidConcurrency".to_string());
    }
    if settings
        .speed_limit_bytes_per_second
        .is_some_and(|limit| !(MIN_SPEED_LIMIT..=MAX_SPEED_LIMIT).contains(&limit))
    {
        return Err("invalidSpeedLimit".to_string());
    }
    if settings
        .browser_for_cookies
        .as_deref()
        .is_some_and(|browser| !SUPPORTED_COOKIE_BROWSERS.contains(&browser))
    {
        return Err("invalidCookieBrowser".to_string());
    }
    let output_directory = PathBuf::from(settings.output_directory.trim());
    if settings.output_directory.trim().is_empty() || !output_directory.is_absolute() {
        return Err("invalidOutputDirectory".to_string());
    }
    Ok(())
}

fn per_attempt_speed_limit(settings: &AppSettings) -> Option<u64> {
    settings
        .speed_limit_bytes_per_second
        .map(|limit| (limit / settings.concurrency as u64).max(1))
}

fn engine_probe_directory(database_path: &Path) -> PathBuf {
    database_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir)
}

fn drain_engine_events(
    runtime: &SchedulerRuntime,
    reservation: &AttemptReservation,
    process: &SupervisedProcess,
    final_output: &mut Option<PathBuf>,
    failure: &mut EngineFailureClassifier,
) {
    apply_engine_lines(
        runtime,
        reservation,
        process.drain_output(),
        final_output,
        failure,
    );
}

fn apply_engine_lines(
    runtime: &SchedulerRuntime,
    reservation: &AttemptReservation,
    lines: impl IntoIterator<Item = ProcessLine>,
    final_output: &mut Option<PathBuf>,
    failure: &mut EngineFailureClassifier,
) {
    for line in lines {
        failure.observe(line.stream, &line.text);
        match parse_engine_line(line.stream, &line.text) {
            Some(EngineEvent::Progress {
                downloaded_bytes,
                total_bytes,
                speed,
                eta,
            }) => {
                let _ = runtime.with_database(|connection| {
                    storage::update_progress(
                        connection,
                        &reservation.task_id,
                        downloaded_bytes,
                        total_bytes,
                        speed,
                        eta,
                    )
                });
            }
            Some(EngineEvent::PostProcessing) => {
                let _ = runtime.with_database(|connection| {
                    storage::mark_postprocessing(connection, &reservation.task_id)
                });
            }
            Some(EngineEvent::OutputFile(path)) => *final_output = Some(path),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{
        engine_probe_directory, per_attempt_speed_limit, reconcile_completed_output_sizes,
        validate_settings,
    };
    use crate::{model::AppSettings, storage};

    fn settings() -> AppSettings {
        AppSettings {
            concurrency: 2,
            speed_limit_bytes_per_second: None,
            browser_for_cookies: None,
            output_directory: std::env::temp_dir().to_string_lossy().into_owned(),
        }
    }

    #[test]
    fn validates_resource_settings() {
        assert!(validate_settings(&settings()).is_ok());

        let mut invalid = settings();
        invalid.concurrency = 0;
        assert_eq!(
            validate_settings(&invalid).unwrap_err(),
            "invalidConcurrency"
        );

        invalid = settings();
        invalid.speed_limit_bytes_per_second = Some(1);
        assert_eq!(
            validate_settings(&invalid).unwrap_err(),
            "invalidSpeedLimit"
        );

        invalid = settings();
        invalid.browser_for_cookies = Some("unsupported".to_string());
        assert_eq!(
            validate_settings(&invalid).unwrap_err(),
            "invalidCookieBrowser"
        );

        invalid = settings();
        invalid.output_directory = "relative/path".to_string();
        assert_eq!(
            validate_settings(&invalid).unwrap_err(),
            "invalidOutputDirectory"
        );
    }

    #[test]
    fn probes_engines_from_app_data_instead_of_the_protected_download_folder() {
        let database_path = PathBuf::from("app-data").join("jivefetch.sqlite3");
        assert_eq!(
            engine_probe_directory(&database_path),
            PathBuf::from("app-data")
        );
    }

    #[test]
    fn divides_the_global_speed_budget_between_execution_slots() {
        let mut configured = settings();
        configured.concurrency = 4;
        configured.speed_limit_bytes_per_second = Some(2 * 1024 * 1024);
        assert_eq!(per_attempt_speed_limit(&configured), Some(512 * 1024));
    }

    #[test]
    fn reconciles_zero_byte_metrics_for_a_completed_output_after_restart() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("output");
        fs::create_dir_all(&output).unwrap();
        let media = output.join("video.webm");
        fs::write(&media, b"complete media").unwrap();

        let connection = storage::open_database(&directory.path().join("queue.sqlite3")).unwrap();
        let task = storage::insert_task(&connection, "https://example.com/video").unwrap();
        connection
            .execute(
                "UPDATE tasks SET state = 'completed', progress = 1.0, output_path = ?1
                 WHERE id = ?2",
                rusqlite::params![media.to_string_lossy(), task.id],
            )
            .unwrap();

        assert_eq!(
            reconcile_completed_output_sizes(&connection, &output),
            Ok(1)
        );
        let repaired = storage::load_task(&connection, &task.id).unwrap();
        assert_eq!(repaired.downloaded_bytes, 14);
        assert_eq!(repaired.total_bytes, Some(14));
        assert_eq!(
            reconcile_completed_output_sizes(&connection, &output),
            Ok(0)
        );
    }
}
