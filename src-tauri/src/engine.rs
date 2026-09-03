use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use crate::{
    model::{EngineInfo, EngineStatus, MediaFormat, MediaProbe},
    process_supervisor::{OutputStream, SupervisedProcess},
};

const PROGRESS_PREFIX: &str = "__JIVEFETCH_PROGRESS__";
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
pub struct ExecutionPlan {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
    pub engine_version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    Progress {
        downloaded_bytes: i64,
        total_bytes: Option<i64>,
        speed: Option<f64>,
        eta: Option<i64>,
    },
    PostProcessing,
    OutputFile(PathBuf),
}

impl EngineRegistry {
    pub fn discover(output_directory: &Path) -> Self {
        Self {
            yt_dlp: discover_binary(&["yt-dlp"], "--version", output_directory),
            ffmpeg: discover_binary(&["ffmpeg"], "-version", output_directory),
        }
    }

    pub fn status(&self) -> EngineStatus {
        EngineStatus {
            ready: self.yt_dlp.is_some() && self.ffmpeg.is_some(),
            yt_dlp: engine_info(self.yt_dlp.as_ref()),
            ffmpeg: engine_info(self.ffmpeg.as_ref()),
        }
    }

    pub fn download_plan(
        &self,
        url: &str,
        output_directory: &Path,
        speed_limit_bytes_per_second: Option<u64>,
        browser_for_cookies: Option<&str>,
        format_selector: Option<&str>,
    ) -> Result<ExecutionPlan, String> {
        let yt_dlp = self
            .yt_dlp
            .as_ref()
            .ok_or_else(|| "ytDlpMissing".to_string())?;
        let ffmpeg = self
            .ffmpeg
            .as_ref()
            .ok_or_else(|| "ffmpegMissing".to_string())?;
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
                "download:{PROGRESS_PREFIX}%(progress.downloaded_bytes|0)s\t%(progress.total_bytes,progress.total_bytes_estimate|0)s\t%(progress.speed|0)s\t%(progress.eta|0)s"
            ),
            "--progress-template".to_string(),
            format!("postprocess:{PHASE_PREFIX}postprocessing"),
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
            args.extend(["--cookies-from-browser".to_string(), browser.to_string()]);
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
        browser_for_cookies: Option<&str>,
    ) -> Result<MediaProbe, String> {
        let yt_dlp = self
            .yt_dlp
            .as_ref()
            .ok_or_else(|| "ytDlpMissing".to_string())?;
        let mut args = vec![
            "--ignore-config".to_string(),
            "--no-playlist".to_string(),
            "--no-color".to_string(),
            "--no-warnings".to_string(),
            "--skip-download".to_string(),
            "--dump-single-json".to_string(),
        ];
        if let Some(browser) = browser_for_cookies {
            args.extend(["--cookies-from-browser".to_string(), browser.to_string()]);
        }
        args.extend(["--".to_string(), url.to_string()]);

        let mut process = SupervisedProcess::spawn_with_output_limits(
            &yt_dlp.path,
            &args,
            output_directory,
            PROBE_MAX_OUTPUT_BYTES,
            8,
        )
        .map_err(|_| "engineSpawnFailed".to_string())?;
        let deadline = Instant::now() + PROBE_TIMEOUT;
        let mut output = String::new();

        loop {
            collect_probe_output(&process, &mut output);
            match process.try_wait() {
                Ok(Some(status)) => {
                    thread::sleep(Duration::from_millis(50));
                    collect_probe_output(&process, &mut output);
                    if !status.success() {
                        return Err("probeFailed".to_string());
                    }
                    return parse_probe_json(&output);
                }
                Ok(None) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(50));
                }
                Ok(None) => {
                    let _ = process.terminate_owned_tree(Duration::from_secs(2));
                    return Err("probeTimedOut".to_string());
                }
                Err(_) => return Err("probeFailed".to_string()),
            }
        }
    }
}

fn collect_probe_output(process: &SupervisedProcess, output: &mut String) {
    for line in process.drain_output() {
        if line.stream == OutputStream::Stdout {
            output.push_str(&line.text);
            output.push('\n');
        }
    }
}

fn parse_probe_json(output: &str) -> Result<MediaProbe, String> {
    let value: serde_json::Value =
        serde_json::from_str(output.trim()).map_err(|_| "probeOutputInvalid".to_string())?;
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
        return Err("noFormats".to_string());
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
    if let Some(payload) = line.strip_prefix(PROGRESS_PREFIX) {
        let mut fields = payload.split('\t');
        let downloaded_bytes = parse_i64(fields.next())?;
        let total = parse_i64(fields.next()).filter(|value| *value > 0);
        let speed = parse_f64(fields.next()).filter(|value| *value > 0.0);
        let eta = parse_i64(fields.next()).filter(|value| *value >= 0);
        return Some(EngineEvent::Progress {
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

    use crate::process_supervisor::OutputStream;

    use super::{
        concise_version, parse_engine_line, parse_probe_json, verified_output_file, EngineBinary,
        EngineEvent, EngineRegistry,
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
    fn parses_machine_progress_and_final_path() {
        assert_eq!(
            parse_engine_line(
                OutputStream::Stdout,
                "__JIVEFETCH_PROGRESS__512\t1024\t128.5\t4"
            ),
            Some(EngineEvent::Progress {
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
            parse_engine_line(OutputStream::Stderr, "__JIVEFETCH_PROGRESS__1\t2\t3\t4"),
            None
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
        let plan = registry
            .download_plan(
                "https://example.com/video?id=1",
                &PathBuf::from("/tmp/downloads"),
                Some(524_288),
                Some("firefox"),
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
        let plan = registry
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
