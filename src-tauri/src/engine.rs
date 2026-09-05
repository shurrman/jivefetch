use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use crate::{
    error::{EngineError, UserErrorCode},
    model::{BrowserCookieSource, EngineInfo, EngineStatus, MediaFormat, MediaProbe},
    process_supervisor::{OutputStream, ProcessLine, SupervisedProcess},
};

const PROGRESS_PREFIX: &str = "__JIVEFETCH_PROGRESS__";
const DOWNLOAD_PLAN_PREFIX: &str = "__JIVEFETCH_DOWNLOAD_PLAN__";
const PHASE_PREFIX: &str = "__JIVEFETCH_PHASE__";
const FILE_PREFIX: &str = "__JIVEFETCH_FILE__";
const PROBE_MAX_OUTPUT_BYTES: usize = 2 * 1024 * 1024;
const PROBE_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Debug, Clone)]
pub struct EngineBinary {
    pub path: PathBuf,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct EngineRegistry {
    pub yt_dlp: Option<EngineBinary>,
    pub ffmpeg: Option<EngineBinary>,
}

#[derive(Debug, Clone)]
pub struct BinaryDiscovery {
    working_directory: PathBuf,
}

#[derive(Debug, Clone)]
pub struct YtDlpExecutor {
    registry: EngineRegistry,
}

pub trait EngineExecutor: Send + Sync {
    fn status(&self) -> EngineStatus;
    fn download_plan(
        &self,
        url: &str,
        output_directory: &Path,
        speed_limit_bytes_per_second: Option<u64>,
        browser_for_cookies: Option<BrowserCookieSource>,
        format_selector: Option<&str>,
    ) -> Result<ExecutionPlan, EngineError>;
    fn probe_formats(
        &self,
        url: &str,
        output_directory: &Path,
        browser_for_cookies: Option<BrowserCookieSource>,
    ) -> Result<MediaProbe, EngineError>;
}

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
    pub engine_version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    DownloadPlan(Vec<DownloadComponent>),
    Progress {
        component_id: String,
        stage: String,
        downloaded_bytes: i64,
        total_bytes: Option<i64>,
        speed: Option<f64>,
        eta: Option<i64>,
    },
    PostProcessing,
    OutputFile(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadComponent {
    pub id: String,
    pub stage: String,
    pub total_bytes: Option<i64>,
}

#[derive(Debug, Default)]
pub struct EngineFailureClassifier {
    browser_cookies_unavailable: bool,
    authentication_required: bool,
    media_unavailable: bool,
    rate_limited: bool,
    http_forbidden: bool,
    format_unavailable: bool,
    network_error: bool,
    permission_denied: bool,
}

impl EngineFailureClassifier {
    pub fn observe(&mut self, stream: OutputStream, line: &str) {
        if stream != OutputStream::Stderr {
            return;
        }

        let line = line.to_ascii_lowercase();
        let mentions_cookies = line.contains("cookie")
            || line.contains("find-generic-password")
            || line.contains("safe storage")
            || line.contains("keyring");
        let cookie_access_failure = line.contains("cannot decrypt")
            || line.contains("could not decrypt")
            || line.contains("failed to decrypt")
            || line.contains("unable to copy")
            || line.contains("could not copy")
            || line.contains("cannot copy")
            || line.contains("not found")
            || line.contains("no key found")
            || line.contains("permission denied")
            || line.contains("operation not permitted")
            || line.contains("failed");
        self.browser_cookies_unavailable |= mentions_cookies && cookie_access_failure;
        self.authentication_required |= [
            "sign in to confirm",
            "login required",
            "authentication required",
            "members-only",
            "confirm you're not a bot",
            "confirm you’re not a bot",
        ]
        .iter()
        .any(|marker| line.contains(marker));
        self.media_unavailable |= [
            "video unavailable",
            "this video is unavailable",
            "private video",
            "has been removed",
        ]
        .iter()
        .any(|marker| line.contains(marker));
        self.rate_limited |= line.contains("http error 429") || line.contains("too many requests");
        self.http_forbidden |= line.contains("http error 403") || line.contains("403: forbidden");
        self.format_unavailable |= line.contains("requested format is not available");
        self.network_error |= [
            "failed to resolve",
            "network is unreachable",
            "connection refused",
            "connection reset",
            "timed out",
            "temporary failure in name resolution",
        ]
        .iter()
        .any(|marker| line.contains(marker));
        self.permission_denied |=
            line.contains("permission denied") || line.contains("operation not permitted");
    }

    pub fn error_code(&self, browser_selected: bool) -> Option<UserErrorCode> {
        if browser_selected && self.browser_cookies_unavailable {
            Some(UserErrorCode::BrowserCookiesUnavailable)
        } else if self.authentication_required {
            Some(UserErrorCode::AuthenticationRequired)
        } else if self.media_unavailable {
            Some(UserErrorCode::MediaUnavailable)
        } else if self.rate_limited {
            Some(UserErrorCode::RateLimited)
        } else if self.http_forbidden {
            Some(UserErrorCode::HttpForbidden)
        } else if self.format_unavailable {
            Some(UserErrorCode::FormatUnavailable)
        } else if self.network_error {
            Some(UserErrorCode::Network)
        } else if self.permission_denied {
            Some(UserErrorCode::PermissionDenied)
        } else if browser_selected {
            Some(UserErrorCode::BrowserCookiesUnavailable)
        } else {
            None
        }
    }
}

impl BinaryDiscovery {
    pub fn new(working_directory: PathBuf) -> Self {
        Self { working_directory }
    }

    pub fn discover(&self) -> EngineRegistry {
        EngineRegistry {
            yt_dlp: discover_binary(&["yt-dlp"], "--version", &self.working_directory),
            ffmpeg: discover_binary(&["ffmpeg"], "-version", &self.working_directory),
        }
    }
}

impl EngineRegistry {
    pub fn status(&self) -> EngineStatus {
        EngineStatus {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            ready: self.yt_dlp.is_some() && self.ffmpeg.is_some(),
            yt_dlp: engine_info(self.yt_dlp.as_ref()),
            ffmpeg: engine_info(self.ffmpeg.as_ref()),
        }
    }
}

impl YtDlpExecutor {
    pub fn new(registry: EngineRegistry) -> Self {
        Self { registry }
    }

    pub fn status(&self) -> EngineStatus {
        self.registry.status()
    }

    pub fn download_plan(
        &self,
        url: &str,
        output_directory: &Path,
        speed_limit_bytes_per_second: Option<u64>,
        browser_for_cookies: Option<BrowserCookieSource>,
        format_selector: Option<&str>,
    ) -> Result<ExecutionPlan, EngineError> {
        let yt_dlp = self
            .registry
            .yt_dlp
            .as_ref()
            .ok_or(EngineError::YtDlpMissing)?;
        let ffmpeg = self
            .registry
            .ffmpeg
            .as_ref()
            .ok_or(EngineError::FfmpegMissing)?;
        let mut args = vec![
            "--ignore-config".to_string(),
            "--no-playlist".to_string(),
            "--newline".to_string(),
            "--no-color".to_string(),
            "--progress".to_string(),
            "--progress-delta".to_string(),
            "0.5".to_string(),
            "--progress-template".to_string(),
            format!(
                "download:{PROGRESS_PREFIX}%(info.format_id|unknown)s\t%(info.vcodec|none)s\t%(info.acodec|none)s\t%(progress.downloaded_bytes|0)s\t%(progress.total_bytes,progress.total_bytes_estimate|0)s\t%(progress.speed|0)s\t%(progress.eta|0)s"
            ),
            "--progress-template".to_string(),
            format!("postprocess:{PHASE_PREFIX}postprocessing"),
            "--print".to_string(),
            format!(
                "before_dl:{DOWNLOAD_PLAN_PREFIX}%(requested_formats.0.format_id,format_id|unknown)s\t%(requested_formats.0.vcodec,vcodec|none)s\t%(requested_formats.0.acodec,acodec|none)s\t%(requested_formats.0.filesize,requested_formats.0.filesize_approx,filesize,filesize_approx|0)s\t%(requested_formats.1.format_id|)s\t%(requested_formats.1.vcodec|none)s\t%(requested_formats.1.acodec|none)s\t%(requested_formats.1.filesize,requested_formats.1.filesize_approx|0)s"
            ),
            "--print".to_string(),
            format!("after_move:{FILE_PREFIX}%(filepath)j"),
            "--paths".to_string(),
            output_directory.to_string_lossy().into_owned(),
            "--output".to_string(),
            "%(title).180B [%(id)s].%(ext)s".to_string(),
            "--continue".to_string(),
            "--no-overwrites".to_string(),
        ];
        if let Some(limit) = speed_limit_bytes_per_second {
            args.extend(["--limit-rate".to_string(), limit.to_string()]);
        }
        if let Some(browser) = browser_for_cookies {
            args.extend([
                "--cookies-from-browser".to_string(),
                browser.as_yt_dlp_arg().to_string(),
            ]);
        }
        args.extend([
            "--format".to_string(),
            format_selector
                .unwrap_or("bestvideo*+bestaudio/best")
                .to_string(),
            "--ffmpeg-location".to_string(),
            ffmpeg.path.to_string_lossy().into_owned(),
            "--".to_string(),
            url.to_string(),
        ]);

        Ok(ExecutionPlan {
            executable: yt_dlp.path.clone(),
            args,
            working_directory: output_directory.to_path_buf(),
            engine_version: yt_dlp.version.clone(),
        })
    }

    pub fn probe_formats(
        &self,
        url: &str,
        output_directory: &Path,
        browser_for_cookies: Option<BrowserCookieSource>,
    ) -> Result<MediaProbe, EngineError> {
        let yt_dlp = self
            .registry
            .yt_dlp
            .as_ref()
            .ok_or(EngineError::YtDlpMissing)?;
        let mut args = vec![
            "--ignore-config".to_string(),
            "--no-playlist".to_string(),
            "--no-color".to_string(),
            "--skip-download".to_string(),
            "--dump-single-json".to_string(),
        ];
        if let Some(browser) = browser_for_cookies {
            args.extend([
                "--cookies-from-browser".to_string(),
                browser.as_yt_dlp_arg().to_string(),
            ]);
        }
        args.extend(["--".to_string(), url.to_string()]);

        let mut process = SupervisedProcess::spawn_with_output_limits(
            &yt_dlp.path,
            &args,
            output_directory,
            PROBE_MAX_OUTPUT_BYTES,
            8,
        )
        .map_err(EngineError::Spawn)?;
        let deadline = Instant::now() + PROBE_TIMEOUT;
        let mut output = String::new();
        let mut failure = EngineFailureClassifier::default();

        loop {
            collect_probe_output(&process, &mut output, &mut failure);
            match process.try_wait() {
                Ok(Some(status)) => {
                    collect_probe_lines(
                        process.drain_output_until_closed(Duration::from_millis(250)),
                        &mut output,
                        &mut failure,
                    );
                    if !status.success() {
                        return Err(EngineError::Classified(
                            failure
                                .error_code(browser_for_cookies.is_some())
                                .unwrap_or(UserErrorCode::ProbeFailed),
                        ));
                    }
                    return parse_probe_json(&output);
                }
                Ok(None) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(50));
                }
                Ok(None) => {
                    let _ = process.terminate_owned_tree(Duration::from_secs(2));
                    return Err(EngineError::ProbeTimedOut);
                }
                Err(error) => return Err(EngineError::Probe(error)),
            }
        }
    }
}

impl EngineExecutor for YtDlpExecutor {
    fn status(&self) -> EngineStatus {
        self.status()
    }

    fn download_plan(
        &self,
        url: &str,
        output_directory: &Path,
        speed_limit_bytes_per_second: Option<u64>,
        browser_for_cookies: Option<BrowserCookieSource>,
        format_selector: Option<&str>,
    ) -> Result<ExecutionPlan, EngineError> {
        self.download_plan(
            url,
            output_directory,
            speed_limit_bytes_per_second,
            browser_for_cookies,
            format_selector,
        )
    }

    fn probe_formats(
        &self,
        url: &str,
        output_directory: &Path,
        browser_for_cookies: Option<BrowserCookieSource>,
    ) -> Result<MediaProbe, EngineError> {
        self.probe_formats(url, output_directory, browser_for_cookies)
    }
}

fn collect_probe_output(
    process: &SupervisedProcess,
    output: &mut String,
    failure: &mut EngineFailureClassifier,
) {
    collect_probe_lines(process.drain_output(), output, failure);
}

fn collect_probe_lines(
    lines: impl IntoIterator<Item = ProcessLine>,
    output: &mut String,
    failure: &mut EngineFailureClassifier,
) {
    for line in lines {
        failure.observe(line.stream, &line.text);
        if line.stream == OutputStream::Stdout {
            output.push_str(&line.text);
            output.push('\n');
        }
    }
}

fn parse_probe_json(output: &str) -> Result<MediaProbe, EngineError> {
    let value: serde_json::Value =
        serde_json::from_str(output.trim()).map_err(EngineError::ProbeOutputInvalid)?;
    let title = value
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Media")
        .to_string();
    let duration = positive_f64(value.get("duration"));
    let mut formats = value
        .get("formats")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(normalize_format)
        .collect::<Vec<_>>();
    formats.sort_by(|left, right| {
        right
            .height
            .cmp(&left.height)
            .then_with(|| compare_optional_f64(right.bitrate_kbps, left.bitrate_kbps))
            .then_with(|| compare_optional_f64(right.fps, left.fps))
            .then_with(|| left.format_id.cmp(&right.format_id))
    });
    formats.dedup_by(|left, right| left.selector == right.selector);
    formats.truncate(100);
    if formats.is_empty() {
        return Err(EngineError::NoFormats);
    }
    Ok(MediaProbe {
        title,
        duration,
        formats,
    })
}

fn normalize_format(value: &serde_json::Value) -> Option<MediaFormat> {
    let format_id = value.get("format_id")?.as_str()?.trim();
    if format_id.is_empty()
        || format_id.len() > 128
        || !format_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
    {
        return None;
    }
    let video_codec = optional_string(value.get("vcodec"));
    if video_codec.as_deref().is_none_or(|codec| codec == "none") {
        return None;
    }
    let has_audio = optional_string(value.get("acodec")).is_some_and(|codec| codec != "none");
    let selector = if has_audio {
        format_id.to_string()
    } else {
        format!("{format_id}+bestaudio/{format_id}/best")
    };
    Some(MediaFormat {
        selector,
        format_id: format_id.to_string(),
        width: positive_u64(value.get("width")),
        height: positive_u64(value.get("height")),
        fps: positive_f64(value.get("fps")),
        video_codec,
        extension: optional_string(value.get("ext")),
        bitrate_kbps: positive_f64(value.get("tbr")).or_else(|| positive_f64(value.get("vbr"))),
        file_size: positive_u64(value.get("filesize"))
            .or_else(|| positive_u64(value.get("filesize_approx"))),
        has_audio,
    })
}

fn optional_string(value: Option<&serde_json::Value>) -> Option<String> {
    value?
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn positive_f64(value: Option<&serde_json::Value>) -> Option<f64> {
    value?
        .as_f64()
        .filter(|value| value.is_finite() && *value > 0.0)
}

fn positive_u64(value: Option<&serde_json::Value>) -> Option<u64> {
    value?.as_u64().filter(|value| *value > 0)
}

fn compare_optional_f64(left: Option<f64>, right: Option<f64>) -> std::cmp::Ordering {
    left.partial_cmp(&right)
        .unwrap_or(std::cmp::Ordering::Equal)
}

pub fn parse_engine_line(stream: OutputStream, line: &str) -> Option<EngineEvent> {
    if stream != OutputStream::Stdout {
        return None;
    }
    if let Some(payload) = line.strip_prefix(DOWNLOAD_PLAN_PREFIX) {
        let fields = payload.split('\t').collect::<Vec<_>>();
        let mut components = Vec::new();
        for component in fields.as_chunks::<4>().0.iter().take(2) {
            let id = component[0].trim();
            if id.is_empty() || id == "NA" {
                continue;
            }
            components.push(DownloadComponent {
                id: id.to_string(),
                stage: media_stage(component[1], component[2]).to_string(),
                total_bytes: parse_i64(Some(component[3])).filter(|value| *value > 0),
            });
        }
        if !components.is_empty() {
            return Some(EngineEvent::DownloadPlan(components));
        }
    }
    if let Some(payload) = line.strip_prefix(PROGRESS_PREFIX) {
        let mut fields = payload.split('\t');
        let component_id = fields.next()?.trim().to_string();
        let video_codec = fields.next()?;
        let audio_codec = fields.next()?;
        let downloaded_bytes = parse_i64(fields.next())?;
        let total = parse_i64(fields.next()).filter(|value| *value > 0);
        let speed = parse_f64(fields.next()).filter(|value| *value > 0.0);
        let eta = parse_i64(fields.next()).filter(|value| *value >= 0);
        return Some(EngineEvent::Progress {
            component_id,
            stage: media_stage(video_codec, audio_codec).to_string(),
            downloaded_bytes,
            total_bytes: total,
            speed,
            eta,
        });
    }
    if line == format!("{PHASE_PREFIX}postprocessing") {
        return Some(EngineEvent::PostProcessing);
    }
    if let Some(payload) = line.strip_prefix(FILE_PREFIX) {
        let path = serde_json::from_str::<String>(payload).unwrap_or_else(|_| payload.to_string());
        return Some(EngineEvent::OutputFile(PathBuf::from(path)));
    }
    None
}

fn media_stage(video_codec: &str, audio_codec: &str) -> &'static str {
    let has_video = !video_codec.trim().is_empty()
        && !matches!(
            video_codec.trim().to_ascii_lowercase().as_str(),
            "none" | "na"
        );
    let has_audio = !audio_codec.trim().is_empty()
        && !matches!(
            audio_codec.trim().to_ascii_lowercase().as_str(),
            "none" | "na"
        );
    match (has_video, has_audio) {
        (true, false) => "video",
        (false, true) => "audio",
        _ => "media",
    }
}

pub fn verified_output_file(path: &Path, output_directory: &Path) -> Option<(String, i64)> {
    let canonical_output = output_directory.canonicalize().ok()?;
    let canonical_path = path.canonicalize().ok()?;
    canonical_path.strip_prefix(canonical_output).ok()?;
    let metadata = canonical_path.metadata().ok()?;
    let size = i64::try_from(metadata.len()).ok()?;
    if !metadata.is_file() || size <= 0 {
        return None;
    }
    Some((canonical_path.to_string_lossy().into_owned(), size))
}

fn parse_i64(value: Option<&str>) -> Option<i64> {
    value?
        .trim()
        .parse::<f64>()
        .ok()
        .map(|number| number as i64)
}

fn parse_f64(value: Option<&str>) -> Option<f64> {
    value?.trim().parse().ok()
}

fn engine_info(binary: Option<&EngineBinary>) -> EngineInfo {
    EngineInfo {
        available: binary.is_some(),
        version: binary.map(|binary| binary.version.clone()),
    }
}

fn discover_binary(
    names: &[&str],
    version_argument: &str,
    working_directory: &Path,
) -> Option<EngineBinary> {
    for path in executable_candidates(names) {
        if !path.is_file() {
            continue;
        }
        if let Some(version) = read_version(&path, version_argument, working_directory) {
            return Some(EngineBinary { path, version });
        }
    }
    None
}

fn executable_candidates(names: &[&str]) -> Vec<PathBuf> {
    let mut directories = env::var_os("PATH")
        .map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();
    for directory in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"] {
        let candidate = PathBuf::from(directory);
        if !directories.contains(&candidate) {
            directories.push(candidate);
        }
    }

    let extensions = executable_extensions();
    let mut candidates = Vec::new();
    for directory in directories {
        for name in names {
            for extension in &extensions {
                let mut file_name = OsString::from(name);
                file_name.push(extension);
                candidates.push(directory.join(file_name));
            }
        }
    }
    candidates
}

#[cfg(windows)]
fn executable_extensions() -> Vec<OsString> {
    let configured = env::var_os("PATHEXT").unwrap_or_else(|| OsString::from(".EXE;.CMD;.BAT"));
    configured
        .to_string_lossy()
        .split(';')
        .map(OsString::from)
        .collect()
}

#[cfg(not(windows))]
fn executable_extensions() -> Vec<OsString> {
    vec![OsString::new()]
}

fn read_version(
    executable: &Path,
    version_argument: &str,
    working_directory: &Path,
) -> Option<String> {
    let args = vec![version_argument.to_string()];
    let mut process = SupervisedProcess::spawn(executable, &args, working_directory).ok()?;
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut lines = Vec::new();

    while Instant::now() < deadline {
        lines.extend(process.drain_output().map(|line| line.text));
        match process.try_wait() {
            Ok(Some(status)) if status.success() => {
                let drain_deadline = Instant::now() + Duration::from_secs(1);
                while Instant::now() < drain_deadline {
                    lines.extend(process.drain_output().map(|line| line.text));
                    if !lines.is_empty() {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                return lines
                    .into_iter()
                    .find(|line| !line.trim().is_empty())
                    .map(|line| concise_version(&line));
            }
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let _ = process.terminate_owned_tree(Duration::ZERO);
    None
}

fn concise_version(line: &str) -> String {
    line.strip_prefix("ffmpeg version ")
        .or_else(|| line.strip_prefix("ffprobe version "))
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or(line)
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{
        error::UserErrorCode, model::BrowserCookieSource, process_supervisor::OutputStream,
    };

    use super::{
        concise_version, parse_engine_line, parse_probe_json, verified_output_file, EngineBinary,
        EngineEvent, EngineFailureClassifier, EngineRegistry, YtDlpExecutor,
    };

    #[test]
    fn keeps_engine_versions_concise_for_the_header() {
        assert_eq!(
            concise_version("ffmpeg version 8.1.2 Copyright (c) 2000-2026"),
            "8.1.2"
        );
        assert_eq!(concise_version("2026.07.04"), "2026.07.04");
    }

    #[test]
    fn classifies_engine_failures_without_persisting_raw_output() {
        let mut cookies = EngineFailureClassifier::default();
        cookies.observe(
            OutputStream::Stderr,
            "WARNING: cannot decrypt v10 cookies: no key found",
        );
        assert_eq!(
            cookies.error_code(true),
            Some(UserErrorCode::BrowserCookiesUnavailable)
        );
        assert_eq!(cookies.error_code(false), None);

        let mut authentication = EngineFailureClassifier::default();
        authentication.observe(
            OutputStream::Stderr,
            "ERROR: Sign in to confirm you’re not a bot",
        );
        assert_eq!(
            authentication.error_code(false),
            Some(UserErrorCode::AuthenticationRequired)
        );

        let mut network = EngineFailureClassifier::default();
        network.observe(
            OutputStream::Stderr,
            "ERROR: Failed to resolve 'www.example.com'",
        );
        assert_eq!(network.error_code(false), Some(UserErrorCode::Network));

        let mut forbidden = EngineFailureClassifier::default();
        forbidden.observe(
            OutputStream::Stderr,
            "ERROR: unable to download video data: HTTP Error 403: Forbidden",
        );
        assert_eq!(
            forbidden.error_code(false),
            Some(UserErrorCode::HttpForbidden)
        );

        assert_eq!(
            EngineFailureClassifier::default().error_code(true),
            Some(UserErrorCode::BrowserCookiesUnavailable)
        );
    }

    #[test]
    fn parses_machine_progress_and_final_path() {
        assert_eq!(
            parse_engine_line(
                OutputStream::Stdout,
                "__JIVEFETCH_PROGRESS__137\tavc1\tnone\t512\t1024\t128.5\t4"
            ),
            Some(EngineEvent::Progress {
                component_id: "137".to_string(),
                stage: "video".to_string(),
                downloaded_bytes: 512,
                total_bytes: Some(1024),
                speed: Some(128.5),
                eta: Some(4),
            })
        );
        assert_eq!(
            parse_engine_line(OutputStream::Stdout, "__JIVEFETCH_FILE__\"/tmp/video.mp4\""),
            Some(EngineEvent::OutputFile(PathBuf::from("/tmp/video.mp4")))
        );
        assert_eq!(
            parse_engine_line(
                OutputStream::Stderr,
                "__JIVEFETCH_PROGRESS__137\tavc1\tnone\t1\t2\t3\t4"
            ),
            None
        );
        assert_eq!(
            parse_engine_line(
                OutputStream::Stdout,
                "__JIVEFETCH_DOWNLOAD_PLAN__137\tavc1\tnone\t1000\t251\tnone\topus\t250"
            ),
            Some(EngineEvent::DownloadPlan(vec![
                super::DownloadComponent {
                    id: "137".to_string(),
                    stage: "video".to_string(),
                    total_bytes: Some(1000),
                },
                super::DownloadComponent {
                    id: "251".to_string(),
                    stage: "audio".to_string(),
                    total_bytes: Some(250),
                },
            ]))
        );
    }

    #[test]
    fn verifies_only_non_empty_regular_output_files_inside_the_download_directory() {
        let output = tempfile::tempdir().unwrap();
        let file = output.path().join("video.webm");
        fs::write(&file, b"media").unwrap();
        assert_eq!(
            verified_output_file(&file, output.path()),
            Some((
                file.canonicalize().unwrap().to_string_lossy().into_owned(),
                5
            ))
        );

        let empty = output.path().join("empty.webm");
        fs::File::create(&empty).unwrap();
        assert_eq!(verified_output_file(&empty, output.path()), None);
        assert_eq!(verified_output_file(output.path(), output.path()), None);

        let outside = tempfile::NamedTempFile::new().unwrap();
        assert_eq!(verified_output_file(outside.path(), output.path()), None);
    }

    #[test]
    fn adds_the_configured_global_speed_limit_without_a_shell() {
        let registry = EngineRegistry {
            yt_dlp: Some(EngineBinary {
                path: PathBuf::from("yt-dlp"),
                version: "test".to_string(),
            }),
            ffmpeg: Some(EngineBinary {
                path: PathBuf::from("ffmpeg"),
                version: "test".to_string(),
            }),
        };
        let plan = YtDlpExecutor::new(registry)
            .download_plan(
                "https://example.com/video?id=1",
                &PathBuf::from("/tmp/downloads"),
                Some(524_288),
                Some(BrowserCookieSource::Firefox),
                None,
            )
            .unwrap();
        assert!(plan
            .args
            .windows(2)
            .any(|args| args == ["--limit-rate", "524288"]));
        assert!(plan
            .args
            .windows(2)
            .any(|args| args == ["--cookies-from-browser", "firefox"]));
        assert!(plan.args.iter().any(|argument| argument == "--progress"));
        assert!(plan
            .args
            .windows(2)
            .any(|args| args == ["--format", "bestvideo*+bestaudio/best"]));
        assert_eq!(plan.args.last().unwrap(), "https://example.com/video?id=1");
    }

    #[test]
    fn uses_a_selected_source_format_and_normalizes_probe_metadata() {
        let registry = EngineRegistry {
            yt_dlp: Some(EngineBinary {
                path: PathBuf::from("yt-dlp"),
                version: "test".to_string(),
            }),
            ffmpeg: Some(EngineBinary {
                path: PathBuf::from("ffmpeg"),
                version: "test".to_string(),
            }),
        };
        let plan = YtDlpExecutor::new(registry)
            .download_plan(
                "https://example.com/video",
                &PathBuf::from("/tmp/downloads"),
                None,
                None,
                Some("137+bestaudio/137/best"),
            )
            .unwrap();
        assert!(plan
            .args
            .windows(2)
            .any(|args| { args == ["--format", "137+bestaudio/137/best"] }));

        let probe = parse_probe_json(
            r#"{
                "title": "Fixture",
                "duration": 12.5,
                "formats": [
                    {"format_id":"audio","vcodec":"none","acodec":"opus","abr":128},
                    {"format_id":"22","width":1280,"height":720,"fps":30,"vcodec":"avc1.64001F","acodec":"mp4a.40.2","ext":"mp4","tbr":1800,"filesize":123456},
                    {"format_id":"137","width":1920,"height":1080,"fps":60,"vcodec":"avc1.640028","acodec":"none","ext":"mp4","vbr":4200,"filesize_approx":654321}
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(probe.title, "Fixture");
        assert_eq!(probe.duration, Some(12.5));
        assert_eq!(probe.formats.len(), 2);
        assert_eq!(probe.formats[0].format_id, "137");
        assert_eq!(probe.formats[0].selector, "137+bestaudio/137/best");
        assert_eq!(probe.formats[0].bitrate_kbps, Some(4200.0));
        assert!(!probe.formats[0].has_audio);
        assert_eq!(probe.formats[1].selector, "22");
        assert!(probe.formats[1].has_audio);
    }
}
