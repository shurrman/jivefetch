use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueueTask {
    pub id: String,
    pub url: String,
    pub state: String,
    pub revision: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub progress: f64,
    pub downloaded_bytes: i64,
    pub total_bytes: Option<i64>,
    pub speed: Option<f64>,
    pub eta: Option<i64>,
    pub output_path: Option<String>,
    pub error_code: Option<String>,
    pub attempt_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub available: bool,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub app_version: String,
    pub ready: bool,
    pub yt_dlp: EngineInfo,
    pub ffmpeg: EngineInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub concurrency: usize,
    pub speed_limit_bytes_per_second: Option<u64>,
    pub browser_for_cookies: Option<String>,
    pub output_directory: String,
}

#[derive(Debug, Clone)]
pub struct AttemptReservation {
    pub task_id: String,
    pub attempt_id: String,
    pub url: String,
    pub format_selector: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaFormat {
    pub selector: String,
    pub format_id: String,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub fps: Option<f64>,
    pub video_codec: Option<String>,
    pub extension: Option<String>,
    pub bitrate_kbps: Option<f64>,
    pub file_size: Option<u64>,
    pub has_audio: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaProbe {
    pub title: String,
    pub duration: Option<f64>,
    pub formats: Vec<MediaFormat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlIntent {
    Pause,
    Stop,
}

impl ControlIntent {
    pub fn transient_state(self) -> &'static str {
        match self {
            Self::Pause => "pausing",
            Self::Stop => "stopping",
        }
    }

    pub fn stable_state(self) -> &'static str {
        match self {
            Self::Pause => "paused",
            Self::Stop => "stopped",
        }
    }
}
