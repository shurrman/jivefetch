use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use jivefetch_lib::process_supervisor::SupervisedProcess;

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn wait_for_change(path: &Path, previous: Option<&str>) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(value) = fs::read_to_string(path) {
            if previous.is_none_or(|previous| previous != value) {
                return value;
            }
        }
        thread::sleep(Duration::from_millis(40));
    }
    panic!("heartbeat did not change: {}", path.display());
}

#[test]
fn terminates_only_the_owned_process_tree() {
    let directory = tempfile::tempdir().unwrap();
    let owned_heartbeat = directory.path().join("owned.txt");
    let unrelated_heartbeat = directory.path().join("unrelated.txt");
    let helper = Path::new(env!("CARGO_BIN_EXE_process-tree-helper"));

    let unrelated = Command::new(helper)
        .arg("child")
        .arg(&unrelated_heartbeat)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _unrelated_guard = ChildGuard(unrelated);

    let args = vec![
        "parent".to_string(),
        owned_heartbeat.to_string_lossy().into_owned(),
    ];
    let mut owned = SupervisedProcess::spawn(helper, &args, directory.path()).unwrap();
    let first_owned = wait_for_change(&owned_heartbeat, None);
    let first_unrelated = wait_for_change(&unrelated_heartbeat, None);

    owned
        .terminate_owned_tree(Duration::from_millis(250))
        .unwrap();
    thread::sleep(Duration::from_millis(120));
    let stopped_owned = fs::read_to_string(&owned_heartbeat).unwrap();
    thread::sleep(Duration::from_millis(180));

    assert_eq!(fs::read_to_string(&owned_heartbeat).unwrap(), stopped_owned);
    let _ = first_owned;
    assert_ne!(
        wait_for_change(&unrelated_heartbeat, Some(&first_unrelated)),
        first_unrelated
    );
}
