use std::{
    io::{self, Read},
    path::Path,
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
    time::{Duration, Instant},
};

use command_group::{CommandGroup, GroupChild};

const MAX_LINE_BYTES: usize = 8 * 1024;
const OUTPUT_QUEUE_DEPTH: usize = 256;

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

impl SupervisedProcess {
    pub fn spawn(executable: &Path, args: &[String], working_directory: &Path) -> io::Result<Self> {
        let mut command = Command::new(executable);
        command
            .args(args)
            .current_dir(working_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear();

        for key in [
            "PATH",
            "HOME",
            "USERPROFILE",
            "TMPDIR",
            "TEMP",
            "TMP",
            "LANG",
        ] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }

        let mut child = command.group_spawn()?;
        let stdout = child.inner().stdout.take();
        let stderr = child.inner().stderr.take();
        let (sender, output) = mpsc::sync_channel(OUTPUT_QUEUE_DEPTH);

        if let Some(stdout) = stdout {
            spawn_reader(stdout, OutputStream::Stdout, sender.clone());
        }
        if let Some(stderr) = stderr {
            spawn_reader(stderr, OutputStream::Stderr, sender);
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

impl Drop for SupervisedProcess {
    fn drop(&mut self) {
        if matches!(self.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn spawn_reader<R>(mut reader: R, stream: OutputStream, sender: SyncSender<ProcessLine>)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut read_buffer = [0_u8; 4096];
        let mut pending = Vec::with_capacity(MAX_LINE_BYTES);

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
                } else if pending.len() < MAX_LINE_BYTES {
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

    use super::{spawn_reader, OutputStream, MAX_LINE_BYTES};

    #[test]
    fn reader_bounds_untrusted_lines() {
        let (sender, receiver) = mpsc::sync_channel(8);
        let oversized = vec![b'x'; MAX_LINE_BYTES * 2];
        spawn_reader(Cursor::new(oversized), OutputStream::Stdout, sender);
        let line = receiver.recv().unwrap();
        assert_eq!(line.text.len(), MAX_LINE_BYTES);
    }
}
