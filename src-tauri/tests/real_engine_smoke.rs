use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use jivefetch_lib::{engine::EngineRegistry, scheduler::SchedulerRuntime};

struct MediaServer {
    address: std::net::SocketAddr,
    stop: Arc<AtomicBool>,
}

impl MediaServer {
    fn start(payload: Vec<u8>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => serve(&mut stream, &payload),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(_) => break,
                }
            }
        });
        Self { address, stop }
    }

    fn url(&self) -> String {
        format!("http://{}/fixture.mp4", self.address)
    }
}

impl Drop for MediaServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn serve(stream: &mut TcpStream, payload: &[u8]) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let mut request = [0_u8; 4096];
    let read = stream.read(&mut request).unwrap_or(0);
    let is_head = request[..read].starts_with(b"HEAD ");
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    let _ = stream.write_all(headers.as_bytes());
    if !is_head {
        let _ = stream.write_all(payload);
    }
}

#[test]
#[ignore = "requires locally installed yt-dlp and ffmpeg"]
fn downloads_local_fixture_through_the_real_scheduler() {
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("output");
    fs::create_dir_all(&output).unwrap();
    let engines = EngineRegistry::discover(&output);
    let ffmpeg = engines
        .ffmpeg
        .expect("ffmpeg is required for this smoke test");
    assert!(
        engines.yt_dlp.is_some(),
        "yt-dlp is required for this smoke test"
    );

    let fixture = directory.path().join("fixture.mp4");
    let status = Command::new(&ffmpeg.path)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=160x90:d=1",
            "-c:v",
            "mpeg4",
            "-y",
        ])
        .arg(&fixture)
        .stdin(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());

    let server = MediaServer::start(fs::read(&fixture).unwrap());
    let runtime = SchedulerRuntime::new(directory.path().join("queue.sqlite3"), output).unwrap();
    let task = runtime.add_task(&server.url()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);

    while Instant::now() < deadline {
        let current = runtime
            .list_tasks()
            .unwrap()
            .into_iter()
            .find(|candidate| candidate.id == task.id)
            .unwrap();
        match current.state.as_str() {
            "completed" => {
                let output_path = current.output_path.expect("completed output path");
                assert!(std::path::Path::new(&output_path).is_file());
                return;
            }
            "failed" => panic!("engine failed with {:?}", current.error_code),
            _ => thread::sleep(Duration::from_millis(100)),
        }
    }
    panic!("real engine smoke test timed out");
}
