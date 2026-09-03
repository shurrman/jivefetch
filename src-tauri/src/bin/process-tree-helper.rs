use std::{env, fs, path::Path, process::Command, thread, time::Duration};

fn heartbeat(path: &Path) -> ! {
    let mut counter = 0_u64;
    loop {
        counter += 1;
        let _ = fs::write(path, counter.to_string());
        thread::sleep(Duration::from_millis(40));
    }
}

fn main() {
    let mut args = env::args_os().skip(1);
    let mode = args.next().expect("mode");
    let path = args.next().expect("heartbeat path");
    if mode == "child" {
        heartbeat(Path::new(&path));
    }

    let executable = env::current_exe().expect("current executable");
    let mut child = Command::new(executable)
        .arg("child")
        .arg(&path)
        .spawn()
        .expect("spawn child");
    let _ = child.wait();
}
