use eyre::Result;
use tracing::instrument;

use super::{LogTrace, Matcher};
use crate::config::PrefixMatcherConfig;

impl Matcher for PrefixMatcherConfig {
    #[instrument(level = "info", skip(self, input))]
    fn apply(&self, input: &str) -> Result<Option<(String, LogTrace)>> {
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
                let trace = LogTrace::new(redirect.clone()).with_step(
                    input.to_string(),
                    "prefix".to_string(),
                    self.prefix.clone(),
                    remainder.to_string(),
                );
                return Ok(Some((redirect, trace)));
            }

            if let Some(matcher) = &self.matcher {
                tracing::info!("prefix matcher delegating to sub matcher");
                if let Some((url, mut trace)) = matcher.apply(remainder)? {
                    // Prepend this matcher's step to the trace
                    let step = super::TraceStep {
                        input: input.to_string(),
                        matcher_type: "prefix".to_string(),
                        matcher_detail: self.prefix.clone(),
                        output: remainder.to_string(),
                    };
                    trace.steps.insert(0, step);
                    return Ok(Some((url, trace)));
                }
            }
        }

        tracing::info!("prefix matcher did not match");
        Ok(None)
    }
}
