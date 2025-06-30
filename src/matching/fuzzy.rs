use eyre::Result;
use tracing::instrument;

use super::{LogTrace, Matcher};
use crate::config::FuzzyMatcherConfig;

impl Matcher for FuzzyMatcherConfig {
    #[instrument(level = "info", skip(self, input))]
    fn apply(&self, input: &str) -> Result<Option<(String, LogTrace)>> {
        tracing::info!(matcher = ?self, input, "running fuzzy matcher");

        let distance = strsim::levenshtein(input, &self.fuzzy);
        if distance as u32 <= self.tolerance {
            if let Some(url) = &self.url {
                let redirect = url.replace("$1", input);
                tracing::info!(%redirect, "fuzzy matcher produced redirect");
                let trace = LogTrace::new(redirect.clone()).with_step(
                    input.to_string(),
                    "fuzzy".to_string(),
                    self.fuzzy.clone(),
                    input.to_string(),
                );
                return Ok(Some((redirect, trace)));
            }
            if let Some(matcher) = &self.matcher {
                tracing::info!("fuzzy matcher delegating to sub matcher");
                if let Some((url, mut trace)) = matcher.apply(input)? {
                    // Prepend this matcher's step to the trace
                    let step = super::TraceStep {
                        input: input.to_string(),
                        matcher_type: "fuzzy".to_string(),
                        matcher_detail: self.fuzzy.clone(),
                        output: input.to_string(),
                    };
                    trace.steps.insert(0, step);
                    return Ok(Some((url, trace)));
                }
            }
        }

        tracing::info!("fuzzy matcher did not match");
        Ok(None)
    }
}
