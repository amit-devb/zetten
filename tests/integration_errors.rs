use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_missing_config() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("ztn").unwrap();
    
    cmd.current_dir(&temp)
        .arg("run")
        .arg("hello")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Configuration file not found"))
        .stderr(predicate::str::contains("Run `ztn init`"));
}

#[test]
fn test_init_creates_file() {
    let temp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("ztn").unwrap();

    cmd.current_dir(&temp)
        .arg("init")
        .arg("python")        .assert()
        .success()
        .stdout(predicate::str::contains("Zetten Initialized Successfully"));

    let config_path = temp.path().join("zetten.toml");
    assert!(config_path.exists());
}

#[test]
fn test_task_not_found_with_suggestion() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("zetten.toml"), r#"
[tasks.build]
cmd = "echo built"
"#).unwrap();

    let mut cmd = Command::cargo_bin("ztn").unwrap();
    cmd.current_dir(&temp)
        .arg("run")
        .arg("buid") // Typo
        .assert()
        .failure()
        .stderr(predicate::str::contains("Task 'buid' not found"))
        .stderr(predicate::str::contains("Did you mean 'build'?"));
}

#[test]
fn test_circular_dependency() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("zetten.toml"), r#"
[tasks.a]
cmd = "echo a"
depends_on = ["b"]

[tasks.b]
cmd = "echo b"
depends_on = ["a"]
"#).unwrap();

    let mut cmd = Command::cargo_bin("ztn").unwrap();
    cmd.current_dir(&temp)
        .arg("run")
        .arg("a")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Circular dependency detected"));
}
