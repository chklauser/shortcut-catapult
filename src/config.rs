use std::path::PathBuf;

use color_eyre::eyre::Result;

/// Determine the config file path.
pub fn config_path(cli: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = cli {
        return Ok(p);
    }
    let xdg = xdg::BaseDirectories::with_prefix("shortcut-catapult");
    let home = xdg
        .get_config_home()
        .ok_or_else(|| color_eyre::eyre::eyre!("missing home directory"))?;
    Ok(home.join("config.yml"))
}
