use eyre::Result;
use tracing::instrument;

use super::Matcher;
use crate::config::FuzzyMatcherConfig;

impl Matcher for FuzzyMatcherConfig {
    #[instrument(level = "info", skip(self, input))]
    fn apply(&self, input: &str) -> Result<Option<String>> {
        tracing::info!(matcher = ?self, input, "running fuzzy matcher");

        let distance = strsim::levenshtein(input, &self.fuzzy);
        if distance as u32 <= self.tolerance {
            if let Some(url) = &self.url {
                let redirect = url.replace("$1", input);
                tracing::info!(%redirect, "fuzzy matcher produced redirect");
                return Ok(Some(redirect));
            }
            if let Some(matcher) = &self.matcher {
                tracing::info!("fuzzy matcher delegating to sub matcher");
                return matcher.apply(input);
            }
        }

        tracing::info!("fuzzy matcher did not match");
        Ok(None)
    }
}
