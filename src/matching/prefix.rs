use eyre::Result;
use tracing::instrument;

use super::Matcher;
use crate::config::PrefixMatcherConfig;

impl Matcher for PrefixMatcherConfig {
    #[instrument(level = "info", skip(self, _input))]
    fn apply(&self, _input: &str) -> Result<Option<String>> {
        unimplemented!("prefix matcher not implemented yet");
    }
}
