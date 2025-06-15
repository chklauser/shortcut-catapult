use color_eyre::eyre::Result;
use tracing::instrument;

#[instrument(level = "trace")]
fn main() -> Result<()> {
    shortcut_catapult::init()?;
    tracing::trace!("main executed");
    Ok(())
}
