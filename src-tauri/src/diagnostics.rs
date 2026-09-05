use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use tracing_subscriber::fmt::MakeWriter;

const LOG_FILE: &str = "jivefetch.log";
const PREVIOUS_LOG_FILE: &str = "jivefetch.log.1";
const MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Clone)]
struct BoundedLogWriter {
    state: Arc<Mutex<BoundedLog>>,
}

struct BoundedLog {
    current_path: PathBuf,
    previous_path: PathBuf,
    file: File,
    bytes_written: u64,
}

struct BoundedLogGuard<'a> {
    state: MutexGuard<'a, BoundedLog>,
}

impl BoundedLog {
    fn open(directory: &Path) -> io::Result<Self> {
        fs::create_dir_all(directory)?;
        let current_path = directory.join(LOG_FILE);
        let previous_path = directory.join(PREVIOUS_LOG_FILE);
        if current_path
            .metadata()
            .map(|metadata| metadata.len() >= MAX_LOG_BYTES)
            .unwrap_or(false)
        {
            rotate_files(&current_path, &previous_path)?;
        }
        let file = open_log(&current_path)?;
        let bytes_written = file.metadata()?.len();
        Ok(Self {
            current_path,
            previous_path,
            file,
            bytes_written,
        })
    }

    fn rotate(&mut self) -> io::Result<()> {
        self.file.flush()?;
        rotate_files(&self.current_path, &self.previous_path)?;
        self.file = open_log(&self.current_path)?;
        self.bytes_written = 0;
        Ok(())
    }
}

impl Write for BoundedLogGuard<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.state.bytes_written.saturating_add(buffer.len() as u64) > MAX_LOG_BYTES {
            self.state.rotate()?;
        }
        let written = self.state.file.write(buffer)?;
        self.state.bytes_written = self.state.bytes_written.saturating_add(written as u64);
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.state.file.flush()
    }
}

impl<'writer> MakeWriter<'writer> for BoundedLogWriter {
    type Writer = BoundedLogGuard<'writer>;

    fn make_writer(&'writer self) -> Self::Writer {
        BoundedLogGuard {
            state: self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        }
    }
}

pub fn init(app_data_directory: &Path) -> io::Result<()> {
    let writer = BoundedLogWriter {
        state: Arc::new(Mutex::new(BoundedLog::open(app_data_directory)?)),
    };
    tracing_subscriber::fmt()
        .json()
        .with_ansi(false)
        .with_target(false)
        .with_current_span(false)
        .with_span_list(false)
        .with_writer(writer)
        .try_init()
        .map_err(io::Error::other)
}

fn open_log(path: &Path) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

fn rotate_files(current: &Path, previous: &Path) -> io::Result<()> {
    if previous.exists() {
        fs::remove_file(previous)?;
    }
    if current.exists() {
        fs::rename(current, previous)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::{BoundedLog, BoundedLogGuard, MAX_LOG_BYTES};

    #[test]
    fn rotates_only_app_owned_bounded_log_files() {
        let directory = tempfile::tempdir().unwrap();
        let mut log = BoundedLog::open(directory.path()).unwrap();
        log.bytes_written = MAX_LOG_BYTES;
        let state = std::sync::Mutex::new(log);
        BoundedLogGuard {
            state: state.lock().unwrap(),
        }
        .write_all(b"next session")
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(directory.path().join("jivefetch.log")).unwrap(),
            "next session"
        );
        assert!(directory.path().join("jivefetch.log.1").exists());
    }
}
