use eyre::Result;
use tracing::instrument;

use super::{LogTrace, Matcher};
use crate::config::ExactMatcherConfig;

impl Matcher for ExactMatcherConfig {
    #[instrument(level = "info", skip(self, input))]
    fn apply(&self, input: &str) -> Result<Option<(String, LogTrace)>> {
        tracing::info!(matcher = ?self, input, "running exact matcher");
        let mut candidate = input;
        if self.trim {
            candidate = candidate.trim();
        }
        let matches = if self.case_sensitive {
            candidate == self.exact
        } else {
            candidate.eq_ignore_ascii_case(&self.exact)
        };
        if matches {
            if let Some(url) = &self.url {
                let redirect = url.replace("$1", candidate);
                tracing::info!(%redirect, "exact matcher produced redirect");
                let trace = LogTrace::new(redirect.clone()).with_step(
                    input.to_string(),
                    "exact".to_string(),
                    self.exact.clone(),
                    candidate.to_string(),
                );
                return Ok(Some((redirect, trace)));
            }
            if let Some(matcher) = &self.matcher {
                tracing::info!("exact matcher delegating to sub matcher");
                if let Some((url, mut trace)) = matcher.apply(candidate)? {
                    // Prepend this matcher's step to the trace
                    let step = super::TraceStep {
                        input: input.to_string(),
                        matcher_type: "exact".to_string(),
                        matcher_detail: self.exact.clone(),
                        output: candidate.to_string(),
                    };
                    trace.steps.insert(0, step);
                    return Ok(Some((url, trace)));
                }
            }
        }
        tracing::info!("exact matcher did not match");
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_redirect() {
        let cfg = ExactMatcherConfig {
            exact: "Hello".into(),
            case_sensitive: false,
            trim: true,
            url: Some("https://example.com?q=$1".into()),
            matcher: None,
        };
        let result = cfg.apply("Hello").unwrap();
        assert_eq!(result.unwrap().0, "https://example.com?q=Hello");
    }
}
