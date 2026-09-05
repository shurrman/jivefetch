use std::{
    env,
    ffi::OsString,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender},
    thread,
    time::{Duration, Instant},
};

use command_group::{CommandGroup, GroupChild};

const DEFAULT_MAX_LINE_BYTES: usize = 8 * 1024;
const DEFAULT_OUTPUT_QUEUE_DEPTH: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub struct ProcessLine {
    pub stream: OutputStream,
    pub text: String,
}

pub struct SupervisedProcess {
    child: GroupChild,
    output: Receiver<ProcessLine>,
}

pub trait OwnedProcess: Send {
    fn id(&self) -> u32;
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>>;
    fn drain_output(&self) -> Vec<ProcessLine>;
    fn drain_output_until_closed(&self, timeout: Duration) -> Vec<ProcessLine>;
    fn terminate_owned_tree(&mut self, grace: Duration) -> io::Result<ExitStatus>;
}

pub trait ProcessSpawner: Send + Sync {
    fn spawn(
        &self,
        executable: &Path,
        args: &[String],
        working_directory: &Path,
    ) -> io::Result<Box<dyn OwnedProcess>>;
}

#[derive(Debug, Default)]
pub struct SystemProcessSpawner;

impl SupervisedProcess {
    pub fn spawn(executable: &Path, args: &[String], working_directory: &Path) -> io::Result<Self> {
        Self::spawn_with_output_limits(
            executable,
            args,
            working_directory,
            DEFAULT_MAX_LINE_BYTES,
            DEFAULT_OUTPUT_QUEUE_DEPTH,
        )
    }

    pub fn spawn_with_output_limits(
        executable: &Path,
        args: &[String],
        working_directory: &Path,
        max_line_bytes: usize,
        output_queue_depth: usize,
    ) -> io::Result<Self> {
        let mut command = Command::new(executable);
        command
            .args(args)
            .current_dir(working_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear();

        command.env("PATH", child_path()?);
        for key in ["HOME", "USERPROFILE", "TMPDIR", "TEMP", "TMP", "LANG"] {
            if let Some(value) = env::var_os(key) {
                command.env(key, value);
            }
        }
        #[cfg(windows)]
        for key in [
            "APPDATA",
            "LOCALAPPDATA",
            "PROGRAMDATA",
            "SYSTEMROOT",
            "WINDIR",
            "PATHEXT",
        ] {
            if let Some(value) = env::var_os(key) {
                command.env(key, value);
            }
        }

        let mut child = command.group_spawn()?;
        let stdout = child.inner().stdout.take();
        let stderr = child.inner().stderr.take();
        let (sender, output) = mpsc::sync_channel(output_queue_depth.max(1));

        if let Some(stdout) = stdout {
            spawn_reader(stdout, OutputStream::Stdout, sender.clone(), max_line_bytes);
        }
        if let Some(stderr) = stderr {
            spawn_reader(stderr, OutputStream::Stderr, sender, max_line_bytes);
        }

        Ok(Self { child, output })
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    pub fn drain_output(&self) -> impl Iterator<Item = ProcessLine> + '_ {
        self.output.try_iter()
    }

    pub fn drain_output_until_closed(&self, timeout: Duration) -> Vec<ProcessLine> {
        let deadline = Instant::now() + timeout;
        let mut lines = Vec::new();
        while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
            match self.output.recv_timeout(remaining) {
                Ok(line) => lines.push(line),
                Err(RecvTimeoutError::Disconnected | RecvTimeoutError::Timeout) => break,
            }
        }
        lines
    }

    pub fn terminate_owned_tree(&mut self, grace: Duration) -> io::Result<ExitStatus> {
        self.request_graceful_stop()?;
        let deadline = Instant::now() + grace;
        while Instant::now() < deadline {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            thread::sleep(Duration::from_millis(50));
        }

        match self.child.kill() {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::InvalidInput => {}
            Err(error) => return Err(error),
        }
        self.child.wait()
    }

    #[cfg(unix)]
    fn request_graceful_stop(&self) -> io::Result<()> {
        use command_group::{Signal, UnixChildExt};
        match self.child.signal(Signal::SIGINT) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::InvalidInput => Ok(()),
            Err(error) => Err(error),
        }
    }

    #[cfg(windows)]
    fn request_graceful_stop(&mut self) -> io::Result<()> {
        match self.child.kill() {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::InvalidInput => Ok(()),
            Err(error) => Err(error),
        }
    }
}

impl OwnedProcess for SupervisedProcess {
    fn id(&self) -> u32 {
        self.id()
    }

    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.try_wait()
    }

    fn drain_output(&self) -> Vec<ProcessLine> {
        self.drain_output().collect()
    }

    fn drain_output_until_closed(&self, timeout: Duration) -> Vec<ProcessLine> {
        self.drain_output_until_closed(timeout)
    }

    fn terminate_owned_tree(&mut self, grace: Duration) -> io::Result<ExitStatus> {
        self.terminate_owned_tree(grace)
    }
}

impl ProcessSpawner for SystemProcessSpawner {
    fn spawn(
        &self,
        executable: &Path,
        args: &[String],
        working_directory: &Path,
    ) -> io::Result<Box<dyn OwnedProcess>> {
        SupervisedProcess::spawn(executable, args, working_directory)
            .map(|process| Box::new(process) as Box<dyn OwnedProcess>)
    }
}

fn child_path() -> io::Result<OsString> {
    let mut directories = env::var_os("PATH")
        .map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();

    #[cfg(target_os = "macos")]
    let standard_directories = [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
    ];
    #[cfg(all(unix, not(target_os = "macos")))]
    let standard_directories = ["/usr/local/bin", "/usr/bin", "/bin"];
    #[cfg(windows)]
    let standard_directories: [&str; 0] = [];

    for directory in standard_directories {
        let directory = PathBuf::from(directory);
        if !directories.contains(&directory) {
            directories.push(directory);
        }
    }

    #[cfg(unix)]
    if let Some(home) = env::var_os("HOME") {
        let directory = PathBuf::from(home).join(".deno/bin");
        if !directories.contains(&directory) {
            directories.push(directory);
        }
    }
    #[cfg(windows)]
    if let Some(profile) = env::var_os("USERPROFILE") {
        let directory = PathBuf::from(profile).join(".deno/bin");
        if !directories.contains(&directory) {
            directories.push(directory);
        }
    }

    env::join_paths(directories).map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
}

impl Drop for SupervisedProcess {
    fn drop(&mut self) {
        if matches!(self.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn spawn_reader<R>(
    mut reader: R,
    stream: OutputStream,
    sender: SyncSender<ProcessLine>,
    max_line_bytes: usize,
) where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut read_buffer = [0_u8; 4096];
        let mut pending = Vec::with_capacity(max_line_bytes);

        loop {
            let read = match reader.read(&mut read_buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(_) => break,
            };

            for byte in &read_buffer[..read] {
                if *byte == b'\n' {
                    send_line(&sender, stream, &pending);
                    pending.clear();
                } else if pending.len() < max_line_bytes {
                    pending.push(*byte);
                }
            }
        }

        if !pending.is_empty() {
            send_line(&sender, stream, &pending);
        }
    });
}

fn send_line(sender: &SyncSender<ProcessLine>, stream: OutputStream, bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes)
        .trim_end_matches('\r')
        .to_string();
    let _ = sender.try_send(ProcessLine { stream, text });
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::mpsc;

    use super::{child_path, spawn_reader, OutputStream, DEFAULT_MAX_LINE_BYTES};

    #[test]
    fn reader_bounds_untrusted_lines() {
        let (sender, receiver) = mpsc::sync_channel(8);
        let oversized = vec![b'x'; DEFAULT_MAX_LINE_BYTES * 2];
        spawn_reader(
            Cursor::new(oversized),
            OutputStream::Stdout,
            sender,
            DEFAULT_MAX_LINE_BYTES,
        );
        let line = receiver.recv().unwrap();
        assert_eq!(line.text.len(), DEFAULT_MAX_LINE_BYTES);
    }

    #[test]
    fn child_path_contains_platform_tool_directories() {
        #[cfg(unix)]
        let directories = std::env::split_paths(&child_path().unwrap()).collect::<Vec<_>>();

        #[cfg(windows)]
        assert!(child_path().is_ok());

        #[cfg(target_os = "macos")]
        assert!(directories.contains(&std::path::PathBuf::from("/opt/homebrew/bin")));
        #[cfg(unix)]
        assert!(directories.contains(&std::path::PathBuf::from("/usr/bin")));
    }
}
