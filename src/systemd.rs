use crate::cli::{InstallArgs, UninstallArgs};
use color_eyre::eyre::{Result, eyre};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

const SERVICE_TEMPLATE: &str = r#"[Unit]
Description=Shortcut Catapult URL redirection service
Requires=shortcut-catapult.socket

[Service]
Type=notify
ExecStart={binary_path} daemon --systemd
StandardOutput=journal
StandardError=journal
Restart=on-failure

[Install]
WantedBy=default.target
"#;

const SOCKET_TEMPLATE: &str = r#"[Unit]
Description=Shortcut Catapult Socket
PartOf=shortcut-catapult.service

[Socket]
ListenStream=127.0.0.1:{port}
Accept=false

[Install]
WantedBy=sockets.target
"#;

/// Get systemd user unit directory  
fn user_unit_dir() -> Result<PathBuf> {
    // Use the standard systemd user unit directory location
    let home = std::env::var("HOME").map_err(|_| eyre!("HOME environment variable not set"))?;
    Ok(PathBuf::from(home).join(".local/share/systemd/user"))
}

/// Get the binary path (argv[0])
fn get_binary_path() -> Result<String> {
    let args: Vec<String> = std::env::args().collect();
    if args.is_empty() {
        return Err(eyre!("Could not determine binary path"));
    }
    Ok(args[0].clone())
}

/// Create systemd unit files
fn create_unit_files(port: u16) -> Result<()> {
    let unit_dir = user_unit_dir()?;
    std::fs::create_dir_all(&unit_dir)?;

    let binary_path = get_binary_path()?;

    // Create service file
    let service_content = SERVICE_TEMPLATE.replace("{binary_path}", &binary_path);
    let service_path = unit_dir.join("shortcut-catapult.service");
    std::fs::write(&service_path, service_content)?;
    info!("Created service file: {}", service_path.display());

    // Create socket file
    let socket_content = SOCKET_TEMPLATE.replace("{port}", &port.to_string());
    let socket_path = unit_dir.join("shortcut-catapult.socket");
    std::fs::write(&socket_path, socket_content)?;
    info!("Created socket file: {}", socket_path.display());

    Ok(())
}

/// Run systemctl command and check for errors
fn run_systemctl(args: &[&str]) -> Result<()> {
    debug!("Running systemctl with args: {:?}", args);
    let output = Command::new("systemctl")
        .args(["--user"])
        .args(args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("systemctl command failed: {}", stderr));
    }

    Ok(())
}

/// Install systemd user service and socket
pub fn install(args: InstallArgs) -> Result<()> {
    info!(
        "Installing systemd user service and socket on port {}",
        args.port
    );

    // Create unit files
    create_unit_files(args.port)?;

    // Reload systemd daemon
    run_systemctl(&["daemon-reload"])?;
    info!("Reloaded systemd daemon");

    // Enable and start socket
    run_systemctl(&["enable", "shortcut-catapult.socket"])?;
    info!("Enabled shortcut-catapult.socket");

    run_systemctl(&["start", "shortcut-catapult.socket"])?;
    info!("Started shortcut-catapult.socket");

    info!("Installation completed successfully");
    Ok(())
}

/// Uninstall systemd user service and socket
pub fn uninstall(_args: UninstallArgs) -> Result<()> {
    info!("Uninstalling systemd user service and socket");

    // Stop and disable socket (ignore errors for idempotency)
    let _ = run_systemctl(&["stop", "shortcut-catapult.socket"]);
    let _ = run_systemctl(&["disable", "shortcut-catapult.socket"]);
    info!("Stopped and disabled shortcut-catapult.socket");

    // Stop and disable service (ignore errors for idempotency)
    let _ = run_systemctl(&["stop", "shortcut-catapult.service"]);
    let _ = run_systemctl(&["disable", "shortcut-catapult.service"]);
    info!("Stopped and disabled shortcut-catapult.service");

    // Remove unit files
    let unit_dir = user_unit_dir()?;
    let service_path = unit_dir.join("shortcut-catapult.service");
    let socket_path = unit_dir.join("shortcut-catapult.socket");

    if service_path.exists() {
        std::fs::remove_file(&service_path)?;
        info!("Removed service file: {}", service_path.display());
    }

    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
        info!("Removed socket file: {}", socket_path.display());
    }

    // Reload systemd daemon
    run_systemctl(&["daemon-reload"])?;
    info!("Reloaded systemd daemon");

    info!("Uninstallation completed successfully");
    Ok(())
}

/// Get socket file descriptors from systemd
pub fn get_systemd_listeners() -> Result<Vec<std::os::unix::io::RawFd>> {
    use std::os::unix::io::RawFd;

    // Check if we're running under systemd
    if std::env::var("LISTEN_PID").is_err() {
        return Err(eyre!("Not running under systemd socket activation"));
    }

    let listen_pid: u32 = std::env::var("LISTEN_PID")
        .map_err(|_| eyre!("LISTEN_PID not set"))?
        .parse()
        .map_err(|_| eyre!("Invalid LISTEN_PID"))?;

    if listen_pid != std::process::id() {
        return Err(eyre!("LISTEN_PID does not match current process"));
    }

    let listen_fds: i32 = std::env::var("LISTEN_FDS")
        .map_err(|_| eyre!("LISTEN_FDS not set"))?
        .parse()
        .map_err(|_| eyre!("Invalid LISTEN_FDS"))?;

    if listen_fds < 1 {
        return Err(eyre!("No listening sockets provided by systemd"));
    }

    // systemd passes file descriptors starting from SD_LISTEN_FDS_START (3)
    const SD_LISTEN_FDS_START: RawFd = 3;
    let fds: Vec<RawFd> = (SD_LISTEN_FDS_START..SD_LISTEN_FDS_START + listen_fds).collect();

    debug!(
        "Got {} file descriptors from systemd: {:?}",
        listen_fds, fds
    );
    Ok(fds)
}

/// Notify systemd that the service is ready
pub fn notify_ready() -> Result<()> {
    match libsystemd::daemon::notify(false, &[libsystemd::daemon::NotifyState::Ready]) {
        Ok(true) => {
            info!("Notified systemd that service is ready");
            Ok(())
        }
        Ok(false) => {
            debug!("systemd notification not sent (not running under systemd)");
            Ok(())
        }
        Err(err) => Err(eyre!("Failed to notify systemd: {}", err)),
    }
}
