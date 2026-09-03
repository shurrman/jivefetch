use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    engine::{parse_engine_line, verified_output_path, EngineEvent, EngineRegistry},
    model::{AttemptReservation, ControlIntent, EngineStatus, QueueTask},
    process_supervisor::SupervisedProcess,
    storage,
};

const MAX_CONCURRENT_DOWNLOADS: usize = 2;
const STOP_GRACE: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct SchedulerRuntime {
    inner: Arc<RuntimeInner>,
}

struct RuntimeInner {
    database_path: PathBuf,
    output_directory: PathBuf,
    engines: EngineRegistry,
    database_writer: Mutex<()>,
    dispatcher: Mutex<()>,
    active: Mutex<HashMap<String, Sender<ControlIntent>>>,
}

impl SchedulerRuntime {
    pub fn new(database_path: PathBuf, output_directory: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&output_directory).map_err(|_| "outputDirectoryError".to_string())?;
        let connection = storage::open_database(&database_path)?;
        storage::recover_interrupted(&connection)?;
        let engines = EngineRegistry::discover(&output_directory);
        Ok(Self {
            inner: Arc::new(RuntimeInner {
                database_path,
                output_directory,
                engines,
                database_writer: Mutex::new(()),
                dispatcher: Mutex::new(()),
                active: Mutex::new(HashMap::new()),
            }),
        })
    }

    pub fn engine_status(&self) -> EngineStatus {
        self.inner
            .engines
            .status(&self.inner.output_directory, MAX_CONCURRENT_DOWNLOADS)
    }

    pub fn list_tasks(&self) -> Result<Vec<QueueTask>, String> {
        self.with_database(storage::list_tasks)
    }

    pub fn add_task(&self, url: &str) -> Result<QueueTask, String> {
        let task = self.with_database(|connection| storage::insert_task(connection, url))?;
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
            if active_count >= MAX_CONCURRENT_DOWNLOADS {
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
    let plan = match runtime
        .inner
        .engines
        .download_plan(&reservation.url, &runtime.inner.output_directory)
    {
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

    loop {
        drain_engine_events(runtime, reservation, &process, &mut final_output);

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
                thread::sleep(Duration::from_millis(50));
                drain_engine_events(runtime, reservation, &process, &mut final_output);
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
        .and_then(|path| verified_output_path(path, &runtime.inner.output_directory));
    if let Some(path) = verified_output.as_deref() {
        let _ = runtime.with_database(|connection| {
            storage::record_output(connection, &reservation.task_id, path)
        });
    }

    let success = exit_success && verified_output.is_some() && !process_error;
    let error_code = if process_error {
        Some("processSupervisorError")
    } else if exit_success && verified_output.is_none() {
        Some("outputMissing")
    } else if !exit_success {
        Some("engineFailed")
    } else {
        None
    };
    let _ = runtime.with_database_mut(|connection| {
        storage::finalize_attempt(connection, reservation, control, success, error_code)
    });
}

fn drain_engine_events(
    runtime: &SchedulerRuntime,
    reservation: &AttemptReservation,
    process: &SupervisedProcess,
    final_output: &mut Option<PathBuf>,
) {
    for line in process.drain_output() {
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
