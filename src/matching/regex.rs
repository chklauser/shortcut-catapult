use eyre::Result;
use tracing::instrument;

use super::Matcher;
use crate::config::RegexMatcherConfig;

impl Matcher for RegexMatcherConfig {
    #[instrument(level = "info", skip(self, _input))]
    fn apply(&self, _input: &str) -> Result<Option<String>> {
        unimplemented!("regex matcher not implemented yet");
    }
}
