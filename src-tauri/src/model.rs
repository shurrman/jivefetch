use serde::Serialize;

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
    pub ready: bool,
    pub yt_dlp: EngineInfo,
    pub ffmpeg: EngineInfo,
    pub output_directory: String,
    pub concurrency: usize,
}

#[derive(Debug, Clone)]
pub struct AttemptReservation {
    pub task_id: String,
    pub attempt_id: String,
    pub url: String,
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
