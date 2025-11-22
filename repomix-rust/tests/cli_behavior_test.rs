use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

#[test]
fn stdout_mode_emits_machine_readable_output() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("hello.txt");
    fs::write(&file_path, "hello").unwrap();

    let output = cargo_bin_cmd!("repomix-rs")
        .arg(dir.path())
        .arg("--style")
        .arg("plain")
        .arg("--stdout")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("File: hello.txt"));
    assert!(!stdout.contains("Repomix v"));
    assert!(!stdout.contains("Packing completed"));
}

#[test]
fn quiet_mode_suppresses_console_noise() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("quiet.txt");
    fs::write(&file_path, "silent").unwrap();
    let output_path = dir.path().join("out.xml");

    let output = cargo_bin_cmd!("repomix-rs")
        .arg(dir.path())
        .arg("--output")
        .arg(&output_path)
        .arg("--quiet")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output_path.exists());

    let written = fs::read_to_string(&output_path).unwrap();
    assert!(!written.is_empty());
}

#[test]
fn version_output_includes_runtime_and_platform() {
    let output = cargo_bin_cmd!("repomix-rs")
        .arg("--version")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
    assert!(stdout.contains("repomix"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    assert!(stdout.contains("runtime"));
    assert!(stdout.contains("platform"));
}
