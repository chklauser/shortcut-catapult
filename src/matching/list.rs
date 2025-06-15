use eyre::Result;
use tracing::instrument;

use super::Matcher;
use crate::config::MatcherConfig;

impl Matcher for Vec<MatcherConfig> {
    #[instrument(level = "info", skip(self, input))]
    fn apply(&self, input: &str) -> Result<Option<String>> {
        tracing::info!(?input, "running list matcher");
        for matcher in self {
            if let Some(result) = matcher.apply(input)? {
                tracing::info!("list matcher got match");
                return Ok(Some(result));
            }
        }
        tracing::info!("list matcher no match");
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::ExactMatcherConfig;
    use crate::config::MatcherConfig;
    use crate::matching::Matcher;

    #[test]
    fn picks_first_match() {
        let m1 = MatcherConfig::Exact(ExactMatcherConfig {
            exact: "One".into(),
            case_sensitive: false,
            trim: true,
            url: Some("https://one.example".into()),
            matcher: None,
        });
        let m2 = MatcherConfig::Exact(ExactMatcherConfig {
            exact: "Two".into(),
            case_sensitive: false,
            trim: true,
            url: Some("https://two.example".into()),
            matcher: None,
        });
        let list = vec![m1, m2];
        let result = list.apply("Two").unwrap();
        assert_eq!(result.unwrap(), "https://two.example");
    }
}
