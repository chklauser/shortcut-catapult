use eyre::Result;
use tracing::instrument;

use super::Matcher;
use crate::config::FuzzyMatcherConfig;

impl Matcher for FuzzyMatcherConfig {
    #[instrument(level = "info", skip(self, _input))]
    fn apply(&self, _input: &str) -> Result<Option<String>> {
        unimplemented!("fuzzy matcher not implemented yet");
    }
}
