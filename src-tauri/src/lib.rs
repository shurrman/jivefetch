use std::{fs, path::PathBuf};

use tauri::{Manager, State};
use url::Url;

pub mod engine;
pub mod model;
pub mod process_supervisor;
pub mod scheduler;
pub mod storage;

use model::{AppSettings, EngineStatus, MediaProbe, QueueTask};
use scheduler::SchedulerRuntime;

const DATABASE_FILE: &str = "jivefetch.sqlite3";
const OUTPUT_DIRECTORY: &str = "JiveFetch";

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

fn validate_format_selector(value: Option<String>) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty()
        || value.len() > 256
        || value.starts_with('-')
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._+-/*".contains(character))
    {
        return Err("invalidFormatSelection".to_string());
    }
    Ok(Some(value.to_string()))
}

#[tauri::command]
fn list_tasks(runtime: State<'_, SchedulerRuntime>) -> Result<Vec<QueueTask>, String> {
    runtime.list_tasks()
}

#[tauri::command]
fn engine_status(runtime: State<'_, SchedulerRuntime>) -> EngineStatus {
    runtime.engine_status()
}

#[tauri::command]
fn get_settings(runtime: State<'_, SchedulerRuntime>) -> Result<AppSettings, String> {
    runtime.settings()
}

#[tauri::command]
fn update_settings(
    settings: AppSettings,
    runtime: State<'_, SchedulerRuntime>,
) -> Result<AppSettings, String> {
    runtime.update_settings(settings)
}

#[tauri::command]
async fn probe_url(
    url: String,
    runtime: State<'_, SchedulerRuntime>,
) -> Result<MediaProbe, String> {
    let url = validate_url(&url)?;
    let runtime = runtime.inner().clone();
    tauri::async_runtime::spawn_blocking(move || runtime.probe_formats(&url))
        .await
        .map_err(|_| "schedulerError".to_string())?
}

#[tauri::command]
fn add_task(
    url: String,
    format_selector: Option<String>,
    runtime: State<'_, SchedulerRuntime>,
) -> Result<QueueTask, String> {
    let format_selector = validate_format_selector(format_selector)?;
    runtime.add_task(&validate_url(&url)?, format_selector.as_deref())
}

#[tauri::command]
fn task_action(
    task_id: String,
    action: String,
    expected_revision: i64,
    runtime: State<'_, SchedulerRuntime>,
) -> Result<QueueTask, String> {
    runtime.task_action(&task_id, &action, expected_revision)
}

#[tauri::command]
fn remove_task(
    task_id: String,
    expected_revision: i64,
    runtime: State<'_, SchedulerRuntime>,
) -> Result<(), String> {
    runtime.remove_task(&task_id, expected_revision)
}

fn runtime_paths(app: &tauri::App) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let app_data = app.path().app_data_dir()?;
    fs::create_dir_all(&app_data)?;
    let output_directory = app.path().download_dir()?.join(OUTPUT_DIRECTORY);
    Ok((app_data.join(DATABASE_FILE), output_directory))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let (database_path, output_directory) = runtime_paths(app)?;
            let runtime = SchedulerRuntime::new(database_path, output_directory)
                .map_err(std::io::Error::other)?;
            runtime.kick();
            app.manage(runtime);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_tasks,
            engine_status,
            get_settings,
            update_settings,
            probe_url,
            add_task,
            task_action,
            remove_task
        ])
        .build(tauri::generate_context!())
        .expect("error while building JiveFetch");

    app.run(|app_handle, event| {
        if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
            app_handle.state::<SchedulerRuntime>().shutdown();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{validate_format_selector, validate_url};

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
    fn accepts_only_bounded_typed_format_selectors() {
        assert_eq!(validate_format_selector(None).unwrap(), None);
        assert_eq!(
            validate_format_selector(Some("137+bestaudio/137/best".to_string())).unwrap(),
            Some("137+bestaudio/137/best".to_string())
        );
        assert!(validate_format_selector(Some("--exec=touch".to_string())).is_err());
        assert!(validate_format_selector(Some("best[height>720]".to_string())).is_err());
    }
}
