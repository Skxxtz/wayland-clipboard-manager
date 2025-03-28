use std::process::{Command, Stdio};

#[test]
fn test_app_start() {
    let mut child = Command::new("cargo")
        .arg("run")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start application");

    std::thread::sleep(std::time::Duration::from_secs(2));

    let _ = child.kill();
}

