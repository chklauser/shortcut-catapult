use eyre::Result;
use tracing::instrument;

use super::Matcher;
use crate::config::PrefixMatcherConfig;

impl Matcher for PrefixMatcherConfig {
    #[instrument(level = "info", skip(self, input))]
    fn apply(&self, input: &str) -> Result<Option<String>> {
        tracing::info!(matcher = ?self, input, "running prefix matcher");

        if input.len() < self.prefix.len() {
            tracing::info!("prefix matcher did not match");
            return Ok(None);
        }

        let candidate_prefix = &input[..self.prefix.len()];
        let matches = if self.case_sensitive {
            candidate_prefix == self.prefix
        } else {
            candidate_prefix.eq_ignore_ascii_case(&self.prefix)
        };

        if matches {
            let remainder = &input[self.prefix.len()..];
            if let Some(url) = &self.url {
                let redirect = url.replace("$1", candidate_prefix).replace("$2", remainder);
                tracing::info!(%redirect, "prefix matcher produced redirect");
                return Ok(Some(redirect));
            }

            if let Some(matcher) = &self.matcher {
                tracing::info!("prefix matcher delegating to sub matcher");
                return matcher.apply(remainder);
            }
        }

        tracing::info!("prefix matcher did not match");
        Ok(None)
    }
}
