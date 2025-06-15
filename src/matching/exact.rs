use eyre::Result;
use tracing::instrument;

use super::Matcher;
use crate::config::ExactMatcherConfig;

impl Matcher for ExactMatcherConfig {
    #[instrument(level = "info", skip(self, input))]
    fn apply(&self, input: &str) -> Result<Option<String>> {
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
                return Ok(Some(redirect));
            }
            if let Some(matcher) = &self.matcher {
                tracing::info!("exact matcher delegating to sub matcher");
                return matcher.apply(candidate);
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
        assert_eq!(result.unwrap(), "https://example.com?q=Hello");
    }
}
