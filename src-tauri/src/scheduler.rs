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
        parse_engine_line, verified_output_file, BinaryDiscovery, EngineEvent, EngineExecutor,
        EngineFailureClassifier, YtDlpExecutor,
    },
    error::{SchedulerError, StorageError, UserErrorCode, UserFacingError},
    model::{AppSettings, AttemptReservation, ControlIntent, EngineStatus, MediaProbe, QueueTask},
    process_supervisor::{OwnedProcess, ProcessLine, ProcessSpawner, SystemProcessSpawner},
    storage,
};

const STOP_GRACE: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct SchedulerRuntime {
    inner: Arc<RuntimeInner>,
}

struct RuntimeInner {
    database_path: PathBuf,
    engines: Arc<dyn EngineExecutor>,
    process_spawner: Arc<dyn ProcessSpawner>,
    settings: Mutex<AppSettings>,
    database_writer: Mutex<()>,
    dispatcher: Mutex<()>,
    active: Mutex<HashMap<String, Sender<ControlIntent>>>,
}

impl SchedulerRuntime {
    pub fn new(database_path: PathBuf, output_directory: PathBuf) -> Result<Self, SchedulerError> {
        let registry = BinaryDiscovery::new(engine_probe_directory(&database_path)).discover();
        Self::new_with_dependencies(
            database_path,
            output_directory,
            Arc::new(YtDlpExecutor::new(registry)),
            Arc::new(SystemProcessSpawner),
        )
    }

    fn new_with_dependencies(
        database_path: PathBuf,
        output_directory: PathBuf,
        engines: Arc<dyn EngineExecutor>,
        process_spawner: Arc<dyn ProcessSpawner>,
    ) -> Result<Self, SchedulerError> {
        let connection = storage::open_database(&database_path)?;
        storage::recover_interrupted(&connection)?;
        let settings = storage::load_settings(&connection, &output_directory)?;
        settings.validate()?;
        let configured_output = PathBuf::from(&settings.output_directory);
        fs::create_dir_all(&configured_output).map_err(SchedulerError::OutputDirectory)?;
        reconcile_completed_output_sizes(&connection, &configured_output)?;
        tracing::info!(
            concurrency = settings.concurrency,
            speed_limit_configured = settings.speed_limit_bytes_per_second.is_some(),
            browser_cookies_configured = settings.browser_for_cookies.is_some(),
            "scheduler initialized"
        );
        Ok(Self {
            inner: Arc::new(RuntimeInner {
                database_path,
                engines,
                process_spawner,
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

    pub fn settings(&self) -> Result<AppSettings, SchedulerError> {
        self.inner
            .settings
            .lock()
            .map(|settings| settings.clone())
            .map_err(|_| SchedulerError::StatePoisoned)
    }

    pub fn update_settings(
        &self,
        mut settings: AppSettings,
    ) -> Result<AppSettings, SchedulerError> {
        settings.output_directory = settings.output_directory.trim().to_string();
        settings.validate()?;
        fs::create_dir_all(&settings.output_directory).map_err(SchedulerError::OutputDirectory)?;
        self.with_database_mut(|connection| storage::save_settings(connection, &settings))?;
        *self
            .inner
            .settings
            .lock()
            .map_err(|_| SchedulerError::StatePoisoned)? = settings.clone();
        tracing::info!(
            concurrency = settings.concurrency,
            speed_limit_configured = settings.speed_limit_bytes_per_second.is_some(),
            browser_cookies_configured = settings.browser_for_cookies.is_some(),
            "settings updated"
        );
        self.kick();
        Ok(settings)
    }

    pub fn list_tasks(&self) -> Result<Vec<QueueTask>, SchedulerError> {
        let output_directory = PathBuf::from(self.settings()?.output_directory);
        let mut tasks = self.with_database(storage::list_tasks)?;
        for task in &mut tasks {
            task.output_available = task.state == "completed"
                && task.output_path.as_deref().is_some_and(|path| {
                    verified_output_file(Path::new(path), &output_directory).is_some()
                });
        }
        Ok(tasks)
    }

    pub fn completed_output_path(&self, task_id: &str) -> Result<PathBuf, SchedulerError> {
        let output_directory = PathBuf::from(self.settings()?.output_directory);
        let task = self.with_database(|connection| storage::load_task(connection, task_id))?;
        if task.state != "completed" {
            return Err(StorageError::OutputMissing.into());
        }
        let stored_path = task
            .output_path
            .as_deref()
            .ok_or(StorageError::OutputMissing)?;
        let (path, _) = verified_output_file(Path::new(stored_path), &output_directory)
            .ok_or(StorageError::OutputMissing)?;
        Ok(PathBuf::from(path))
    }

    pub fn probe_formats(&self, url: &str) -> Result<MediaProbe, SchedulerError> {
        let settings = self.settings()?;
        Ok(self.inner.engines.probe_formats(
            url,
            Path::new(&settings.output_directory),
            settings.browser_cookie_source()?,
        )?)
    }

    pub fn add_task(
        &self,
        url: &str,
        format_selector: Option<&str>,
    ) -> Result<QueueTask, SchedulerError> {
        let task = self.with_database(|connection| {
            storage::insert_task_with_format(connection, url, format_selector)
        })?;
        tracing::info!(task_id = %task.id, "task enqueued");
        self.kick();
        Ok(task)
    }

    pub fn task_action(
        &self,
        task_id: &str,
        action: &str,
        expected_revision: i64,
    ) -> Result<QueueTask, SchedulerError> {
        let outcome = self.with_database_mut(|connection| {
            storage::apply_action(connection, task_id, action, expected_revision)
        })?;

        let mut task = outcome.task;
        if let Some(control) = outcome.control {
            let sender = self
                .inner
                .active
                .lock()
                .map_err(|_| SchedulerError::StatePoisoned)?
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
        tracing::info!(task_id, action, state = %task.state, "task action applied");
        Ok(task)
    }

    pub fn remove_task(&self, task_id: &str, expected_revision: i64) -> Result<(), SchedulerError> {
        self.with_database(|connection| {
            storage::remove_task(connection, task_id, expected_revision)
        })?;
        tracing::info!(task_id, "task removed");
        Ok(())
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
                Ok(None) => return,
                Err(error) => {
                    tracing::error!(error_code = %error.user_code(), "task reservation failed");
                    return;
                }
            };
            tracing::info!(
                task_id = %reservation.task_id,
                attempt_id = %reservation.attempt_id,
                "attempt reserved"
            );
            let (control_sender, control_receiver) = mpsc::channel();
            if let Ok(mut active) = self.inner.active.lock() {
                active.insert(reservation.task_id.clone(), control_sender);
            } else {
                let _ = self.with_database_mut(|connection| {
                    storage::fail_reservation(connection, &reservation, UserErrorCode::Scheduler)
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
        operation: impl FnOnce(&rusqlite::Connection) -> Result<T, StorageError>,
    ) -> Result<T, SchedulerError> {
        let _writer = self
            .inner
            .database_writer
            .lock()
            .map_err(|_| SchedulerError::StatePoisoned)?;
        let connection = storage::open_database(&self.inner.database_path)?;
        Ok(operation(&connection)?)
    }

    fn with_database_mut<T>(
        &self,
        operation: impl FnOnce(&mut rusqlite::Connection) -> Result<T, StorageError>,
    ) -> Result<T, SchedulerError> {
        let _writer = self
            .inner
            .database_writer
            .lock()
            .map_err(|_| SchedulerError::StatePoisoned)?;
        let mut connection = storage::open_database(&self.inner.database_path)?;
        Ok(operation(&mut connection)?)
    }
}

fn run_attempt(
    runtime: &SchedulerRuntime,
    reservation: &AttemptReservation,
    control_receiver: mpsc::Receiver<ControlIntent>,
) {
    let settings = match runtime.settings() {
        Ok(settings) => settings,
        Err(error) => {
            tracing::error!(
                task_id = %reservation.task_id,
                attempt_id = %reservation.attempt_id,
                error_code = %error.user_code(),
                "attempt could not read settings"
            );
            let _ = runtime.with_database_mut(|connection| {
                storage::fail_reservation(connection, reservation, UserErrorCode::Scheduler)
            });
            return;
        }
    };
    let output_directory = PathBuf::from(&settings.output_directory);
    let speed_limit = per_attempt_speed_limit(&settings);
    let browser_for_cookies = match settings.browser_cookie_source() {
        Ok(source) => source,
        Err(error) => {
            let code = error.user_code();
            tracing::error!(
                task_id = %reservation.task_id,
                attempt_id = %reservation.attempt_id,
                error_code = %code,
                "attempt has invalid cookie settings"
            );
            let _ = runtime.with_database_mut(|connection| {
                storage::fail_reservation(connection, reservation, code)
            });
            return;
        }
    };
    let plan = match runtime.inner.engines.download_plan(
        &reservation.url,
        &output_directory,
        speed_limit,
        browser_for_cookies,
        reservation.format_selector.as_deref(),
    ) {
        Ok(plan) => plan,
        Err(error) => {
            let code = error.user_code();
            tracing::error!(
                task_id = %reservation.task_id,
                attempt_id = %reservation.attempt_id,
                error_code = %code,
                "engine plan rejected"
            );
            let _ = runtime.with_database_mut(|connection| {
                storage::fail_reservation(connection, reservation, code)
            });
            return;
        }
    };

    let mut process = match runtime.inner.process_spawner.spawn(
        &plan.executable,
        &plan.args,
        &plan.working_directory,
    ) {
        Ok(process) => process,
        Err(_) => {
            tracing::error!(
                task_id = %reservation.task_id,
                attempt_id = %reservation.attempt_id,
                error_code = %UserErrorCode::EngineSpawnFailed,
                "engine spawn failed"
            );
            let _ = runtime.with_database_mut(|connection| {
                storage::fail_reservation(connection, reservation, UserErrorCode::EngineSpawnFailed)
            });
            return;
        }
    };

    if let Err(error) = runtime.with_database(|connection| {
        storage::mark_started(connection, reservation, process.id(), &plan.engine_version)
    }) {
        tracing::error!(
            task_id = %reservation.task_id,
            attempt_id = %reservation.attempt_id,
            error_code = %error.user_code(),
            "started process could not be persisted"
        );
        let _ = process.terminate_owned_tree(STOP_GRACE);
        let _ = runtime.with_database_mut(|connection| {
            storage::fail_reservation(connection, reservation, UserErrorCode::Storage)
        });
        return;
    }
    tracing::info!(
        task_id = %reservation.task_id,
        attempt_id = %reservation.attempt_id,
        pid = process.id(),
        engine_version = %plan.engine_version,
        browser_cookies = browser_for_cookies.is_some(),
        "engine process started"
    );

    let mut control = None;
    let mut final_output = None;
    let mut exit_success = false;
    let mut process_error = false;
    let mut storage_error = false;
    let mut failure = EngineFailureClassifier::default();
    let mut progress = ProgressAccumulator::default();

    loop {
        drain_engine_events(
            runtime,
            reservation,
            process.as_ref(),
            &mut final_output,
            &mut failure,
            &mut progress,
        );

        if let Ok(intent) = control_receiver.try_recv() {
            control = Some(intent);
            tracing::info!(
                task_id = %reservation.task_id,
                attempt_id = %reservation.attempt_id,
                intent = ?intent,
                "attempt control requested"
            );
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
                    &mut progress,
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
        Some(UserErrorCode::Storage)
    } else if process_error {
        Some(UserErrorCode::ProcessSupervisor)
    } else if exit_success && verified_output.is_none() {
        Some(UserErrorCode::OutputMissing)
    } else if !exit_success {
        Some(
            failure
                .error_code(settings.browser_for_cookies.is_some())
                .unwrap_or(UserErrorCode::EngineFailed),
        )
    } else {
        None
    };
    if let Err(error) = runtime.with_database_mut(|connection| {
        storage::finalize_attempt(connection, reservation, control, success, error_code)
    }) {
        tracing::error!(
            task_id = %reservation.task_id,
            attempt_id = %reservation.attempt_id,
            error_code = %error.user_code(),
            "attempt result could not be persisted"
        );
    } else {
        tracing::info!(
            task_id = %reservation.task_id,
            attempt_id = %reservation.attempt_id,
            success,
            error_code = error_code.map(UserErrorCode::as_str),
            "attempt settled"
        );
    }
}

fn reconcile_completed_output_sizes(
    connection: &rusqlite::Connection,
    output_directory: &Path,
) -> Result<usize, StorageError> {
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
    process: &dyn OwnedProcess,
    final_output: &mut Option<PathBuf>,
    failure: &mut EngineFailureClassifier,
    progress: &mut ProgressAccumulator,
) {
    apply_engine_lines(
        runtime,
        reservation,
        process.drain_output(),
        final_output,
        failure,
        progress,
    );
}

fn apply_engine_lines(
    runtime: &SchedulerRuntime,
    reservation: &AttemptReservation,
    lines: impl IntoIterator<Item = ProcessLine>,
    final_output: &mut Option<PathBuf>,
    failure: &mut EngineFailureClassifier,
    progress: &mut ProgressAccumulator,
) {
    for line in lines {
        failure.observe(line.stream, &line.text);
        match parse_engine_line(line.stream, &line.text) {
            Some(EngineEvent::DownloadPlan(components)) => progress.register_plan(components),
            Some(EngineEvent::Progress {
                component_id,
                stage,
                downloaded_bytes,
                total_bytes,
                speed,
                eta,
            }) => {
                let aggregate = progress.observe(
                    component_id,
                    stage,
                    downloaded_bytes,
                    total_bytes,
                    speed,
                    eta,
                );
                let _ = runtime.with_database(|connection| {
                    storage::update_progress(
                        connection,
                        &reservation.task_id,
                        &storage::ProgressUpdate {
                            progress: aggregate.progress,
                            downloaded_bytes: aggregate.downloaded_bytes,
                            total_bytes: aggregate.total_bytes,
                            speed: aggregate.speed,
                            eta: aggregate.eta,
                            download_stage: &aggregate.stage,
                        },
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

#[derive(Debug, Default)]
struct ProgressAccumulator {
    components: HashMap<String, ComponentProgress>,
    maximum_progress: f64,
}

#[derive(Debug, Default)]
struct ComponentProgress {
    downloaded_bytes: i64,
    total_bytes: Option<i64>,
}

#[derive(Debug, PartialEq)]
struct AggregatedProgress {
    progress: f64,
    downloaded_bytes: i64,
    total_bytes: Option<i64>,
    speed: Option<f64>,
    eta: Option<i64>,
    stage: String,
}

impl ProgressAccumulator {
    fn register_plan(&mut self, components: Vec<crate::engine::DownloadComponent>) {
        for component in components {
            let entry = self.components.entry(component.id).or_default();
            entry.total_bytes = component.total_bytes.or(entry.total_bytes);
        }
    }

    fn observe(
        &mut self,
        component_id: String,
        stage: String,
        downloaded_bytes: i64,
        total_bytes: Option<i64>,
        speed: Option<f64>,
        component_eta: Option<i64>,
    ) -> AggregatedProgress {
        let component = self.components.entry(component_id).or_default();
        component.downloaded_bytes = component.downloaded_bytes.max(downloaded_bytes.max(0));
        component.total_bytes = total_bytes.or(component.total_bytes);

        let downloaded_bytes = self.components.values().fold(0_i64, |total, component| {
            total.saturating_add(component.downloaded_bytes)
        });
        let total_bytes = self
            .components
            .values()
            .map(|component| component.total_bytes)
            .try_fold(0_i64, |total, component| {
                component.map(|value| total.saturating_add(value))
            })
            .filter(|total| *total > 0);
        let measured_progress = total_bytes
            .map(|total| downloaded_bytes as f64 / total as f64)
            .unwrap_or(self.maximum_progress);
        self.maximum_progress = self
            .maximum_progress
            .max(measured_progress)
            .clamp(0.0, 0.99);
        let eta = match (total_bytes, speed.filter(|value| *value > 0.0)) {
            (Some(total), Some(bytes_per_second)) => Some(
                (((total - downloaded_bytes).max(0) as f64 / bytes_per_second).ceil() as i64)
                    .max(0),
            ),
            _ => component_eta,
        };

        AggregatedProgress {
            progress: self.maximum_progress,
            downloaded_bytes,
            total_bytes,
            speed,
            eta,
            stage,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io, path::PathBuf, sync::Arc};

    use super::{
        engine_probe_directory, per_attempt_speed_limit, reconcile_completed_output_sizes,
        ProgressAccumulator, SchedulerRuntime,
    };
    use crate::{
        engine::{DownloadComponent, EngineExecutor, ExecutionPlan},
        error::{EngineError, ValidationError},
        model::{AppSettings, BrowserCookieSource, EngineInfo, EngineStatus, MediaProbe},
        process_supervisor::{OwnedProcess, ProcessSpawner},
        storage,
    };

    struct FakeEngine;

    impl EngineExecutor for FakeEngine {
        fn status(&self) -> EngineStatus {
            EngineStatus {
                app_version: "test".to_string(),
                ready: true,
                yt_dlp: EngineInfo {
                    available: true,
                    version: Some("test".to_string()),
                },
                ffmpeg: EngineInfo {
                    available: true,
                    version: Some("test".to_string()),
                },
            }
        }

        fn download_plan(
            &self,
            _url: &str,
            _output_directory: &std::path::Path,
            _speed_limit_bytes_per_second: Option<u64>,
            _browser_for_cookies: Option<BrowserCookieSource>,
            _format_selector: Option<&str>,
        ) -> Result<ExecutionPlan, EngineError> {
            unreachable!("no task is dispatched by this test")
        }

        fn probe_formats(
            &self,
            _url: &str,
            _output_directory: &std::path::Path,
            _browser_for_cookies: Option<BrowserCookieSource>,
        ) -> Result<MediaProbe, EngineError> {
            unreachable!("no probe is requested by this test")
        }
    }

    struct FakeSpawner;

    impl ProcessSpawner for FakeSpawner {
        fn spawn(
            &self,
            _executable: &std::path::Path,
            _args: &[String],
            _working_directory: &std::path::Path,
        ) -> io::Result<Box<dyn OwnedProcess>> {
            Err(io::Error::other("no process expected"))
        }
    }

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
        assert!(settings().validate().is_ok());

        let mut invalid = settings();
        invalid.concurrency = 0;
        assert_eq!(invalid.validate(), Err(ValidationError::InvalidConcurrency));

        invalid = settings();
        invalid.speed_limit_bytes_per_second = Some(1);
        assert_eq!(invalid.validate(), Err(ValidationError::InvalidSpeedLimit));

        invalid = settings();
        invalid.browser_for_cookies = Some("unsupported".to_string());
        assert_eq!(
            invalid.validate(),
            Err(ValidationError::InvalidCookieBrowser)
        );

        invalid = settings();
        invalid.output_directory = "relative/path".to_string();
        assert_eq!(
            invalid.validate(),
            Err(ValidationError::InvalidOutputDirectory)
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
    fn accepts_narrow_engine_and_process_test_doubles() {
        let directory = tempfile::tempdir().unwrap();
        let runtime = SchedulerRuntime::new_with_dependencies(
            directory.path().join("queue.sqlite3"),
            directory.path().join("output"),
            Arc::new(FakeEngine),
            Arc::new(FakeSpawner),
        )
        .unwrap();
        assert!(runtime.engine_status().ready);
    }

    #[test]
    fn divides_the_global_speed_budget_between_execution_slots() {
        let mut configured = settings();
        configured.concurrency = 4;
        configured.speed_limit_bytes_per_second = Some(2 * 1024 * 1024);
        assert_eq!(per_attempt_speed_limit(&configured), Some(512 * 1024));
    }

    #[test]
    fn aggregates_video_and_audio_without_resetting_overall_progress() {
        let mut progress = ProgressAccumulator::default();
        progress.register_plan(vec![
            DownloadComponent {
                id: "video".to_string(),
                stage: "video".to_string(),
                total_bytes: Some(800),
            },
            DownloadComponent {
                id: "audio".to_string(),
                stage: "audio".to_string(),
                total_bytes: Some(200),
            },
        ]);

        let video = progress.observe(
            "video".to_string(),
            "video".to_string(),
            800,
            Some(800),
            Some(100.0),
            Some(0),
        );
        assert_eq!(video.progress, 0.8);
        assert_eq!(video.downloaded_bytes, 800);
        assert_eq!(video.total_bytes, Some(1000));

        let audio = progress.observe(
            "audio".to_string(),
            "audio".to_string(),
            40,
            Some(200),
            Some(20.0),
            Some(8),
        );
        assert_eq!(audio.progress, 0.84);
        assert_eq!(audio.downloaded_bytes, 840);
        assert_eq!(audio.total_bytes, Some(1000));
        assert_eq!(audio.eta, Some(8));
        assert_eq!(audio.stage, "audio");
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
            reconcile_completed_output_sizes(&connection, &output).unwrap(),
            1
        );
        let repaired = storage::load_task(&connection, &task.id).unwrap();
        assert_eq!(repaired.downloaded_bytes, 14);
        assert_eq!(repaired.total_bytes, Some(14));
        assert_eq!(
            reconcile_completed_output_sizes(&connection, &output).unwrap(),
            0
        );
    }

    #[test]
    fn exposes_completed_output_only_while_the_verified_file_exists() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("output");
        fs::create_dir_all(&output).unwrap();
        let database = directory.path().join("queue.sqlite3");
        let runtime = SchedulerRuntime::new_with_dependencies(
            database.clone(),
            output.clone(),
            Arc::new(FakeEngine),
            Arc::new(FakeSpawner),
        )
        .unwrap();
        let media = output.join("video.mp4");
        fs::write(&media, b"complete media").unwrap();
        let connection = storage::open_database(&database).unwrap();
        let task = storage::insert_task(&connection, "https://example.com/video").unwrap();
        connection
            .execute(
                "UPDATE tasks SET state = 'completed', progress = 1.0, output_path = ?1
                 WHERE id = ?2",
                rusqlite::params![media.to_string_lossy(), task.id],
            )
            .unwrap();

        let listed = runtime.list_tasks().unwrap();
        assert!(listed[0].output_available);
        assert_eq!(
            runtime.completed_output_path(&task.id).unwrap(),
            media.canonicalize().unwrap()
        );

        fs::remove_file(&media).unwrap();
        assert!(!runtime.list_tasks().unwrap()[0].output_available);
        assert!(runtime.completed_output_path(&task.id).is_err());
    }
}
