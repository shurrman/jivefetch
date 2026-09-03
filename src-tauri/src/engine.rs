use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use crate::{
    model::{EngineInfo, EngineStatus},
    process_supervisor::{OutputStream, SupervisedProcess},
};

const PROGRESS_PREFIX: &str = "__JIVEFETCH_PROGRESS__";
const PHASE_PREFIX: &str = "__JIVEFETCH_PHASE__";
const FILE_PREFIX: &str = "__JIVEFETCH_FILE__";

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

    pub fn status(&self, output_directory: &Path, concurrency: usize) -> EngineStatus {
        EngineStatus {
            ready: self.yt_dlp.is_some() && self.ffmpeg.is_some(),
            yt_dlp: engine_info(self.yt_dlp.as_ref()),
            ffmpeg: engine_info(self.ffmpeg.as_ref()),
            output_directory: output_directory.to_string_lossy().into_owned(),
            concurrency,
        }
    }

    pub fn download_plan(
        &self,
        url: &str,
        output_directory: &Path,
    ) -> Result<ExecutionPlan, String> {
        let yt_dlp = self
            .yt_dlp
            .as_ref()
            .ok_or_else(|| "ytDlpMissing".to_string())?;
        let ffmpeg = self
            .ffmpeg
            .as_ref()
            .ok_or_else(|| "ffmpegMissing".to_string())?;
        let args = vec![
            "--ignore-config".to_string(),
            "--no-playlist".to_string(),
            "--newline".to_string(),
            "--no-color".to_string(),
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
            "--format".to_string(),
            "bestvideo*+bestaudio/best".to_string(),
            "--ffmpeg-location".to_string(),
            ffmpeg.path.to_string_lossy().into_owned(),
            "--".to_string(),
            url.to_string(),
        ];

        Ok(ExecutionPlan {
            executable: yt_dlp.path.clone(),
            args,
            working_directory: output_directory.to_path_buf(),
            engine_version: yt_dlp.version.clone(),
        })
    }
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

pub fn verified_output_path(path: &Path, output_directory: &Path) -> Option<String> {
    let canonical_output = output_directory.canonicalize().ok()?;
    let canonical_path = path.canonicalize().ok()?;
    canonical_path
        .strip_prefix(canonical_output)
        .ok()
        .map(|_| canonical_path.to_string_lossy().into_owned())
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
                let drain_deadline = Instant::now() + Duration::from_millis(150);
                while Instant::now() < drain_deadline {
                    lines.extend(process.drain_output().map(|line| line.text));
                    if !lines.is_empty() {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                return lines.into_iter().find(|line| !line.trim().is_empty());
            }
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let _ = process.terminate_owned_tree(Duration::ZERO);
    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::process_supervisor::OutputStream;

    use super::{parse_engine_line, EngineEvent};

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
}
