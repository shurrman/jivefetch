use std::{path::Path, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::ValidationError;

pub const DEFAULT_CONCURRENCY: usize = 2;
pub const MAX_CONFIGURED_CONCURRENCY: usize = 64;
pub const MIN_SPEED_LIMIT: u64 = 1024;
pub const MAX_SPEED_LIMIT: u64 = 10 * 1024 * 1024 * 1024;

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
    pub download_stage: Option<String>,
    pub output_path: Option<String>,
    pub output_available: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserCookieSource {
    Brave,
    Chrome,
    Chromium,
    Edge,
    Firefox,
    Opera,
    Safari,
    Vivaldi,
    Whale,
}

impl BrowserCookieSource {
    pub const fn as_yt_dlp_arg(self) -> &'static str {
        match self {
            Self::Brave => "brave",
            Self::Chrome => "chrome",
            Self::Chromium => "chromium",
            Self::Edge => "edge",
            Self::Firefox => "firefox",
            Self::Opera => "opera",
            Self::Safari => "safari",
            Self::Vivaldi => "vivaldi",
            Self::Whale => "whale",
        }
    }
}

impl FromStr for BrowserCookieSource {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "brave" => Ok(Self::Brave),
            "chrome" => Ok(Self::Chrome),
            "chromium" => Ok(Self::Chromium),
            "edge" => Ok(Self::Edge),
            "firefox" => Ok(Self::Firefox),
            "opera" => Ok(Self::Opera),
            "safari" => Ok(Self::Safari),
            "vivaldi" => Ok(Self::Vivaldi),
            "whale" => Ok(Self::Whale),
            _ => Err(ValidationError::InvalidCookieBrowser),
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !(1..=MAX_CONFIGURED_CONCURRENCY).contains(&self.concurrency) {
            return Err(ValidationError::InvalidConcurrency);
        }
        if self
            .speed_limit_bytes_per_second
            .is_some_and(|limit| !(MIN_SPEED_LIMIT..=MAX_SPEED_LIMIT).contains(&limit))
        {
            return Err(ValidationError::InvalidSpeedLimit);
        }
        self.browser_cookie_source()?;
        let output_directory = self.output_directory.trim();
        if output_directory.is_empty() || !Path::new(output_directory).is_absolute() {
            return Err(ValidationError::InvalidOutputDirectory);
        }
        Ok(())
    }

    pub fn browser_cookie_source(&self) -> Result<Option<BrowserCookieSource>, ValidationError> {
        self.browser_for_cookies
            .as_deref()
            .map(BrowserCookieSource::from_str)
            .transpose()
    }
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
