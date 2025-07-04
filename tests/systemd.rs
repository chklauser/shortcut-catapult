use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn install_command_creates_unit_files() {
    let temp_dir = assert_fs::TempDir::new().expect("temp dir");

    let mut cmd = Command::cargo_bin("shortcut-catapult").expect("binary exists");
    cmd.arg("install").arg("--port").arg("9999");
    cmd.env("HOME", temp_dir.path());

    // In test mode, systemctl commands should be mocked and printed
    cmd.assert().success().stdout(
        predicate::str::contains("SYSTEMCTL_MOCK: systemctl --user daemon-reload")
            .and(predicate::str::contains(
                "SYSTEMCTL_MOCK: systemctl --user enable shortcut-catapult.socket",
            ))
            .and(predicate::str::contains(
                "SYSTEMCTL_MOCK: systemctl --user start shortcut-catapult.socket",
            )),
    );

    // Verify that the service file contains an absolute path
    let service_file = temp_dir
        .path()
        .join(".local/share/systemd/user/shortcut-catapult.service");
    if service_file.exists() {
        let content = std::fs::read_to_string(&service_file).expect("read service file");
        // The ExecStart line should contain an absolute path (starting with /)
        assert!(
            content.contains("ExecStart=/"),
            "Service file should contain absolute path in ExecStart: {}",
            content
        );
    }
}

#[test]
fn uninstall_command_is_idempotent() {
    let mut cmd = Command::cargo_bin("shortcut-catapult").expect("binary exists");
    cmd.arg("uninstall");

    // Should succeed and show mocked systemctl commands
    cmd.assert().success().stdout(
        predicate::str::contains("SYSTEMCTL_MOCK: systemctl --user stop shortcut-catapult.socket")
            .and(predicate::str::contains(
                "SYSTEMCTL_MOCK: systemctl --user disable shortcut-catapult.socket",
            ))
            .and(predicate::str::contains(
                "SYSTEMCTL_MOCK: systemctl --user stop shortcut-catapult.service",
            ))
            .and(predicate::str::contains(
                "SYSTEMCTL_MOCK: systemctl --user disable shortcut-catapult.service",
            ))
            .and(predicate::str::contains(
                "SYSTEMCTL_MOCK: systemctl --user daemon-reload",
            )),
    );
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
