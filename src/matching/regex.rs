use eyre::Result;
use regex::RegexBuilder;
use tracing::instrument;

use super::Matcher;
use crate::config::RegexMatcherConfig;

impl Matcher for RegexMatcherConfig {
    #[instrument(level = "info", skip(self, input))]
    fn apply(&self, input: &str) -> Result<Option<String>> {
        tracing::info!(matcher = ?self, input, "running regex matcher");

        let regex = RegexBuilder::new(&self.regex)
            .case_insensitive(!self.case_sensitive)
            .build()?;

        let Some(caps) = regex.captures(input) else {
            tracing::info!("regex matcher did not match");
            return Ok(None);
        };

        // Determine the value forwarded to the sub matcher or used for $1 placeholder
        let matched = caps.get(0).map(|m| m.as_str()).unwrap_or("");
        let candidate = if let Some(template) = &self.match_with {
            substitute_template(template, &caps)
        } else {
            matched.to_string()
        };

        if let Some(url) = &self.url {
            let mut redirect = url.clone();
            redirect = substitute_template(&redirect, &caps);
            tracing::info!(%redirect, "regex matcher produced redirect");
            return Ok(Some(redirect));
        }

        if let Some(matcher) = &self.matcher {
            tracing::info!("regex matcher delegating to sub matcher");
            return matcher.apply(&candidate);
        }

        tracing::info!("regex matcher did not match");
        Ok(None)
    }
}

fn substitute_template(template: &str, caps: &regex::Captures) -> String {
    let mut result = template.to_string();
    for i in 0..caps.len() {
        if let Some(m) = caps.get(i) {
            let placeholder = format!("${}", i);
            result = result.replace(&placeholder, m.as_str());
        }
    }
    result
}
