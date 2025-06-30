use assert_cmd::Command;
use assert_fs::fixture::NamedTempFile;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SIMPLE_CONFIG: &str = "match:\n  exact: Hello\n  url: https://example.com?q=$1\n";

const CHAINED_CONFIG: &str = "match:
  regex: '^animals/(.*)'
  match-with: '$1'
  match:
    fuzzy: dog
    tolerance: 2
    url: https://animals.com/$1
";

fn run_apply(config: &str, input: &str) -> assert_cmd::assert::Assert {
    let file = NamedTempFile::new("config.yml").expect("temp file");
    file.write_str(config).expect("write config");
    let mut cmd = Command::cargo_bin("shortcut-catapult").expect("binary exists");
    cmd.arg("--config").arg(file.path()).arg("apply");
    cmd.write_stdin(input);
    cmd.assert()
}

#[test]
fn simple_match_shows_log_trace() {
    let assert = run_apply(SIMPLE_CONFIG, "Hello");
    assert
        .success()
        .stdout(predicate::eq("https://example.com?q=Hello"))
        .stderr(predicate::str::contains(
            "OK: Hello + exact(Hello) => https://example.com?q=Hello",
        ));
}

#[test]
fn chained_match_shows_complete_trace() {
    let assert = run_apply(CHAINED_CONFIG, "animals/dag");
    assert
        .success()
        .stdout(predicate::eq("https://animals.com/dag"))
        .stderr(predicate::str::contains(
            "OK: animals/dag + regex(^animals/(.*)) => dag + fuzzy(dog) => https://animals.com/dag",
        ));
}

#[test]
fn no_match_shows_no_trace() {
    let assert = run_apply(SIMPLE_CONFIG, "World");
    assert
        .code(2) // Exit code 2 for no match
        .stdout(predicate::eq(""))
        .stderr(predicate::str::contains("OK:").not());
}
