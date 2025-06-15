use eyre::Result;

pub trait Matcher {
    /// Applies this matcher to the provided `input` URL.
    ///
    /// Returns `Ok(Some(url))` if the matcher accepts the input and wants to
    /// redirect to `url`. Returns `Ok(None)` if the matcher does not match.
    fn apply(&self, input: &str) -> Result<Option<String>>;
}

mod exact;
mod fuzzy;
mod list;
mod prefix;
mod regex;

use crate::config::MatcherConfig;
use tracing::instrument;

impl Matcher for MatcherConfig {
    #[instrument(level = "info", skip(self, input))]
    fn apply(&self, input: &str) -> Result<Option<String>> {
        match self {
            MatcherConfig::Exact(cfg) => cfg.apply(input),
            MatcherConfig::Prefix(cfg) => cfg.apply(input),
            MatcherConfig::Fuzzy(cfg) => cfg.apply(input),
            MatcherConfig::Regex(cfg) => cfg.apply(input),
            MatcherConfig::List(list) => list.apply(input),
        }
    }
}

impl Matcher for Box<MatcherConfig> {
    #[instrument(level = "info", skip(self, input))]
    fn apply(&self, input: &str) -> Result<Option<String>> {
        self.as_ref().apply(input)
    }
}
