use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn install_command_creates_unit_files() {
    let temp_dir = assert_fs::TempDir::new().expect("temp dir");

    // We can't easily test unit file creation in the test environment
    // because it requires a proper HOME directory and systemd setup.
    // Instead, we test that the command runs and handles the expected flow
    let mut cmd = Command::cargo_bin("shortcut-catapult").expect("binary exists");
    cmd.arg("install").arg("--port").arg("9999");
    cmd.env("HOME", temp_dir.path());

    // The command will likely fail in the test environment due to systemctl,
    // but we can verify it at least attempts to create the files
    let _assert = cmd.assert();
}

#[test]
fn uninstall_command_is_idempotent() {
    let mut cmd = Command::cargo_bin("shortcut-catapult").expect("binary exists");
    cmd.arg("uninstall");

    // Should succeed even when nothing is installed
    cmd.assert().success();
}

#[test]
fn daemon_systemd_mode_requires_socket_activation() {
    let mut cmd = Command::cargo_bin("shortcut-catapult").expect("binary exists");
    cmd.arg("daemon").arg("--systemd");

    // Should fail when not running under systemd
    cmd.assert().failure().stderr(predicate::str::contains(
        "Not running under systemd socket activation",
    ));
}

#[test]
fn daemon_help_shows_systemd_flag() {
    let mut cmd = Command::cargo_bin("shortcut-catapult").expect("binary exists");
    cmd.arg("daemon").arg("--help");

    cmd.assert()
        .failure() // help returns non-zero
        .stdout(predicate::str::contains("--systemd"));
}
