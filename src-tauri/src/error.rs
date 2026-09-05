use std::{io, time::SystemTimeError};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserErrorCode {
    InvalidUrl,
    UnsupportedScheme,
    MissingHost,
    InvalidFormatSelection,
    InvalidConcurrency,
    InvalidSpeedLimit,
    InvalidCookieBrowser,
    InvalidOutputDirectory,
    OutputDirectory,
    TaskNotFound,
    RevisionConflict,
    InvalidAction,
    StopBeforeRemove,
    Storage,
    Clock,
    DatabaseTooNew,
    YtDlpMissing,
    FfmpegMissing,
    EngineSpawnFailed,
    EngineFailed,
    BrowserCookiesUnavailable,
    AuthenticationRequired,
    MediaUnavailable,
    RateLimited,
    HttpForbidden,
    FormatUnavailable,
    Network,
    PermissionDenied,
    OutputMissing,
    OpenOutputFailed,
    ProcessSupervisor,
    Scheduler,
    ProbeFailed,
    ProbeTimedOut,
    ProbeOutputInvalid,
    NoFormats,
}

impl UserErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidUrl => "invalidUrl",
            Self::UnsupportedScheme => "unsupportedScheme",
            Self::MissingHost => "missingHost",
            Self::InvalidFormatSelection => "invalidFormatSelection",
            Self::InvalidConcurrency => "invalidConcurrency",
            Self::InvalidSpeedLimit => "invalidSpeedLimit",
            Self::InvalidCookieBrowser => "invalidCookieBrowser",
            Self::InvalidOutputDirectory => "invalidOutputDirectory",
            Self::OutputDirectory => "outputDirectoryError",
            Self::TaskNotFound => "taskNotFound",
            Self::RevisionConflict => "revisionConflict",
            Self::InvalidAction => "invalidAction",
            Self::StopBeforeRemove => "stopBeforeRemove",
            Self::Storage => "storageError",
            Self::Clock => "clockError",
            Self::DatabaseTooNew => "databaseTooNew",
            Self::YtDlpMissing => "ytDlpMissing",
            Self::FfmpegMissing => "ffmpegMissing",
            Self::EngineSpawnFailed => "engineSpawnFailed",
            Self::EngineFailed => "engineFailed",
            Self::BrowserCookiesUnavailable => "browserCookiesUnavailable",
            Self::AuthenticationRequired => "authenticationRequired",
            Self::MediaUnavailable => "mediaUnavailable",
            Self::RateLimited => "rateLimited",
            Self::HttpForbidden => "httpForbidden",
            Self::FormatUnavailable => "formatUnavailable",
            Self::Network => "networkError",
            Self::PermissionDenied => "permissionDenied",
            Self::OutputMissing => "outputMissing",
            Self::OpenOutputFailed => "openOutputFailed",
            Self::ProcessSupervisor => "processSupervisorError",
            Self::Scheduler => "schedulerError",
            Self::ProbeFailed => "probeFailed",
            Self::ProbeTimedOut => "probeTimedOut",
            Self::ProbeOutputInvalid => "probeOutputInvalid",
            Self::NoFormats => "noFormats",
        }
    }
}

impl std::fmt::Display for UserErrorCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub trait UserFacingError {
    fn user_code(&self) -> UserErrorCode;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("concurrency is outside the supported range")]
    InvalidConcurrency,
    #[error("speed limit is outside the supported range")]
    InvalidSpeedLimit,
    #[error("browser cookie source is unsupported")]
    InvalidCookieBrowser,
    #[error("output directory must be an absolute non-empty path")]
    InvalidOutputDirectory,
}

impl UserFacingError for ValidationError {
    fn user_code(&self) -> UserErrorCode {
        match self {
            Self::InvalidConcurrency => UserErrorCode::InvalidConcurrency,
            Self::InvalidSpeedLimit => UserErrorCode::InvalidSpeedLimit,
            Self::InvalidCookieBrowser => UserErrorCode::InvalidCookieBrowser,
            Self::InvalidOutputDirectory => UserErrorCode::InvalidOutputDirectory,
        }
    }
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("SQLite operation failed: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("system clock is before the Unix epoch: {0}")]
    Clock(#[from] SystemTimeError),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("task does not exist")]
    TaskNotFound,
    #[error("task revision changed")]
    RevisionConflict,
    #[error("task action is invalid for its current state")]
    InvalidAction,
    #[error("an active task must be stopped before removal")]
    StopBeforeRemove,
    #[error("the expected output file is missing or empty")]
    OutputMissing,
    #[error("database schema is newer than this application")]
    DatabaseTooNew,
}

impl UserFacingError for StorageError {
    fn user_code(&self) -> UserErrorCode {
        match self {
            Self::Database(_) => UserErrorCode::Storage,
            Self::Clock(_) => UserErrorCode::Clock,
            Self::Validation(error) => error.user_code(),
            Self::TaskNotFound => UserErrorCode::TaskNotFound,
            Self::RevisionConflict => UserErrorCode::RevisionConflict,
            Self::InvalidAction => UserErrorCode::InvalidAction,
            Self::StopBeforeRemove => UserErrorCode::StopBeforeRemove,
            Self::OutputMissing => UserErrorCode::OutputMissing,
            Self::DatabaseTooNew => UserErrorCode::DatabaseTooNew,
        }
    }
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("yt-dlp is unavailable")]
    YtDlpMissing,
    #[error("FFmpeg is unavailable")]
    FfmpegMissing,
    #[error("engine process could not be spawned: {0}")]
    Spawn(#[source] io::Error),
    #[error("engine probe failed: {0}")]
    Probe(#[source] io::Error),
    #[error("engine probe exceeded its deadline")]
    ProbeTimedOut,
    #[error("engine probe returned invalid JSON: {0}")]
    ProbeOutputInvalid(#[source] serde_json::Error),
    #[error("engine probe returned no usable formats")]
    NoFormats,
    #[error("engine reported {0}")]
    Classified(UserErrorCode),
}

impl UserFacingError for EngineError {
    fn user_code(&self) -> UserErrorCode {
        match self {
            Self::YtDlpMissing => UserErrorCode::YtDlpMissing,
            Self::FfmpegMissing => UserErrorCode::FfmpegMissing,
            Self::Spawn(_) => UserErrorCode::EngineSpawnFailed,
            Self::Probe(_) => UserErrorCode::ProbeFailed,
            Self::ProbeTimedOut => UserErrorCode::ProbeTimedOut,
            Self::ProbeOutputInvalid(_) => UserErrorCode::ProbeOutputInvalid,
            Self::NoFormats => UserErrorCode::NoFormats,
            Self::Classified(code) => *code,
        }
    }
}

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("scheduler state lock is poisoned")]
    StatePoisoned,
    #[error("output directory could not be created: {0}")]
    OutputDirectory(#[source] io::Error),
    #[error("blocking scheduler operation could not be joined")]
    JoinFailed,
}

impl UserFacingError for SchedulerError {
    fn user_code(&self) -> UserErrorCode {
        match self {
            Self::Storage(error) => error.user_code(),
            Self::Engine(error) => error.user_code(),
            Self::Validation(error) => error.user_code(),
            Self::StatePoisoned | Self::JoinFailed => UserErrorCode::Scheduler,
            Self::OutputDirectory(_) => UserErrorCode::OutputDirectory,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum InputError {
    #[error("URL is invalid")]
    InvalidUrl,
    #[error("URL scheme is unsupported")]
    UnsupportedScheme,
    #[error("URL has no host")]
    MissingHost,
    #[error("format selector is invalid")]
    InvalidFormatSelection,
}

impl UserFacingError for InputError {
    fn user_code(&self) -> UserErrorCode {
        match self {
            Self::InvalidUrl => UserErrorCode::InvalidUrl,
            Self::UnsupportedScheme => UserErrorCode::UnsupportedScheme,
            Self::MissingHost => UserErrorCode::MissingHost,
            Self::InvalidFormatSelection => UserErrorCode::InvalidFormatSelection,
        }
    }
}
