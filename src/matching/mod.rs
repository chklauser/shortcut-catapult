use eyre::Result;
use std::fmt;

/// Represents a single step in the matching trace
#[derive(Debug, Clone)]
pub struct TraceStep {
    input: String,
    matcher_type: String,
    matcher_detail: String,
    output: String,
}

/// Represents the complete trace of successful matches
#[derive(Debug, Clone)]
pub struct LogTrace {
    steps: Vec<TraceStep>,
    final_url: String,
}

impl LogTrace {
    pub fn new(final_url: String) -> Self {
        Self {
            steps: Vec::new(),
            final_url,
        }
    }

    pub fn with_step(
        mut self,
        input: String,
        matcher_type: String,
        matcher_detail: String,
        output: String,
    ) -> Self {
        self.steps.push(TraceStep {
            input,
            matcher_type,
            matcher_detail,
            output,
        });
        self
    }
}

impl fmt::Display for LogTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.steps.is_empty() {
            write!(f, "{}", self.final_url)?;
        } else {
            for (i, step) in self.steps.iter().enumerate() {
                if i == 0 {
                    write!(
                        f,
                        "{} + {}({})",
                        step.input, step.matcher_type, step.matcher_detail
                    )?;
                } else {
                    write!(f, " + {}({})", step.matcher_type, step.matcher_detail)?;
                }
                if i < self.steps.len() - 1 {
                    write!(f, " => {}", step.output)?;
                }
            }
            write!(f, " => {}", self.final_url)?;
        }

        Ok(())
    }
}

pub trait Matcher {
    /// Applies this matcher to the provided `input` URL.
    ///
    /// Returns `Ok(Some((url, trace)))` if the matcher accepts the input and wants to
    /// redirect to `url` with the given trace. Returns `Ok(None)` if the matcher does not match.
    fn apply(&self, input: &str) -> Result<Option<(String, LogTrace)>>;
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
    fn apply(&self, input: &str) -> Result<Option<(String, LogTrace)>> {
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
    fn apply(&self, input: &str) -> Result<Option<(String, LogTrace)>> {
        self.as_ref().apply(input)
    }
}
