use assert_cmd::Command;
use assert_fs::fixture::NamedTempFile;
use assert_fs::prelude::*;
use predicates::prelude::*;

const EXACT_CONFIG: &str = "match:\n  exact: Hello\n  url: https://example.com?q=$1\n";
const LIST_CONFIG: &str =
    "match:\n- exact: One\n  url: https://one.example\n- exact: Two\n  url: https://two.example\n";

fn run_apply(config: &str, arg: Option<&str>, stdin: Option<&str>) -> assert_cmd::assert::Assert {
    let file = NamedTempFile::new("config.yml").expect("temp file");
    file.write_str(config).expect("write config");
    let mut cmd = Command::cargo_bin("shortcut-catapult").expect("binary exists");
    cmd.arg("--config").arg(file.path()).arg("apply");
    if let Some(a) = arg {
        cmd.arg(a);
    }
    if let Some(input) = stdin {
        cmd.write_stdin(input);
    }
    cmd.assert()
}

#[test]
fn command_line_match_outputs_redirect() {
    run_apply(EXACT_CONFIG, Some("Hello"), None)
        .success()
        .stdout(predicate::eq("https://example.com?q=Hello"));
}

#[test]
fn stdin_dash_match_outputs_redirect() {
    run_apply(EXACT_CONFIG, Some("-"), Some("Hello"))
        .success()
        .stdout(predicate::eq("https://example.com?q=Hello"));
}

#[test]
fn stdin_implicit_match_outputs_redirect() {
    run_apply(EXACT_CONFIG, None, Some("Hello"))
        .success()
        .stdout(predicate::eq("https://example.com?q=Hello"));
}

#[test]
fn no_match_exit_code_two() {
    run_apply(EXACT_CONFIG, Some("World"), None)
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty());
}

#[test]
fn list_picks_first_match() {
    run_apply(LIST_CONFIG, Some("Two"), None)
        .success()
        .stdout(predicate::eq("https://two.example"));
}
