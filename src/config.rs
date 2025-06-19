use color_eyre::eyre::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

fn default_case_sensitive() -> bool {
    false
}
fn default_trim() -> bool {
    true
}
fn default_tolerance() -> u32 {
    3
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Config {
    #[serde(rename = "match")]
    pub matcher: MatcherConfig,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum MatcherConfig {
    Exact(ExactMatcherConfig),
    Prefix(PrefixMatcherConfig),
    Fuzzy(FuzzyMatcherConfig),
    Regex(RegexMatcherConfig),
    List(Vec<MatcherConfig>),
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct ExactMatcherConfig {
    pub exact: String,
    #[serde(default = "default_case_sensitive", rename = "case-sensitive")]
    pub case_sensitive: bool,
    #[serde(default = "default_trim")]
    pub trim: bool,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(rename = "match")]
    #[serde(default)]
    pub matcher: Option<Box<MatcherConfig>>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct PrefixMatcherConfig {
    pub prefix: String,
    #[serde(default = "default_case_sensitive", rename = "case-sensitive")]
    pub case_sensitive: bool,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(rename = "match")]
    #[serde(default)]
    pub matcher: Option<Box<MatcherConfig>>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct FuzzyMatcherConfig {
    pub fuzzy: String,
    #[serde(default = "default_tolerance")]
    pub tolerance: u32,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(rename = "match")]
    #[serde(default)]
    pub matcher: Option<Box<MatcherConfig>>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct RegexMatcherConfig {
    pub regex: String,
    #[serde(default = "default_case_sensitive", rename = "case-sensitive")]
    pub case_sensitive: bool,
    #[serde(rename = "match-with")]
    #[serde(default)]
    pub match_with: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(rename = "match")]
    #[serde(default)]
    pub matcher: Option<Box<MatcherConfig>>,
}

impl Config {
    pub fn parse(cfg: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(cfg)
    }
}

impl std::str::FromStr for Config {
    type Err = serde_yaml::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_yaml::from_str(s)
    }
}

/// Determine the config file path.
pub fn config_path(cli: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = cli {
        return Ok(p);
    }
    let xdg = xdg::BaseDirectories::with_prefix("shortcut-catapult");
    let home = xdg
        .get_config_home()
        .ok_or_else(|| color_eyre::eyre::eyre!("missing home directory"))?;
    Ok(home.join("config.yml"))
}

/// Read the configuration file synchronously.
pub fn read(config_path: &std::path::Path) -> Result<String> {
    std::fs::read_to_string(config_path).wrap_err_with(|| {
        format!(
            "failed to read configuration file at {}",
            config_path.display()
        )
    })
}

/// Read the configuration file asynchronously.
pub async fn read_async(config_path: &std::path::Path) -> Result<String> {
    tokio::fs::read_to_string(config_path)
        .await
        .wrap_err_with(|| {
            format!(
                "failed to read configuration file at {}",
                config_path.display()
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_defaults() {
        let input = "\
match:\n  exact: Armadillo\n  url: https://google.com?q=$1\n";
        let cfg = Config::parse(input).unwrap();
        let expected = Config {
            matcher: MatcherConfig::Exact(ExactMatcherConfig {
                exact: "Armadillo".into(),
                case_sensitive: false,
                trim: true,
                url: Some("https://google.com?q=$1".into()),
                matcher: None,
            }),
        };
        assert_eq!(cfg, expected);
    }

    #[test]
    fn list_matcher() {
        let input = "\
match:\n- exact: Elephant\n  url: https://kagi.com?q=Elephant\n- exact: Lion\n  url: https://bing.com?q=Lion\n";
        let cfg = Config::parse(input).unwrap();
        let expected = Config {
            matcher: MatcherConfig::List(vec![
                MatcherConfig::Exact(ExactMatcherConfig {
                    exact: "Elephant".into(),
                    case_sensitive: false,
                    trim: true,
                    url: Some("https://kagi.com?q=Elephant".into()),
                    matcher: None,
                }),
                MatcherConfig::Exact(ExactMatcherConfig {
                    exact: "Lion".into(),
                    case_sensitive: false,
                    trim: true,
                    url: Some("https://bing.com?q=Lion".into()),
                    matcher: None,
                }),
            ]),
        };
        assert_eq!(cfg, expected);
    }

    #[test]
    fn regex_matcher() {
        let input = "\
match:\n  regex: (\\w+)\\.txt$\n  url: https://file.drive/$1.txt\n";
        let cfg = Config::parse(input).unwrap();
        let expected = Config {
            matcher: MatcherConfig::Regex(RegexMatcherConfig {
                regex: "(\\w+)\\.txt$".into(),
                case_sensitive: false,
                match_with: None,
                url: Some("https://file.drive/$1.txt".into()),
                matcher: None,
            }),
        };
        assert_eq!(cfg, expected);
    }

    #[test]
    fn prefix_with_submatcher() {
        let input = "\
match:\n  prefix: animals/\n  match:\n    exact: bear\n    url: https://bears.org\n";
        let cfg = Config::parse(input).unwrap();
        let expected = Config {
            matcher: MatcherConfig::Prefix(PrefixMatcherConfig {
                prefix: "animals/".into(),
                case_sensitive: false,
                url: None,
                matcher: Some(Box::new(MatcherConfig::Exact(ExactMatcherConfig {
                    exact: "bear".into(),
                    case_sensitive: false,
                    trim: true,
                    url: Some("https://bears.org".into()),
                    matcher: None,
                }))),
            }),
        };
        assert_eq!(cfg, expected);
    }

    #[test]
    fn fuzzy_default_tolerance() {
        let input = "\
match:\n  fuzzy: Elephant\n  url: https://heavy.animal\n";
        let cfg = Config::parse(input).unwrap();
        let expected = Config {
            matcher: MatcherConfig::Fuzzy(FuzzyMatcherConfig {
                fuzzy: "Elephant".into(),
                tolerance: 3,
                url: Some("https://heavy.animal".into()),
                matcher: None,
            }),
        };
        assert_eq!(cfg, expected);
    }
}
