use assert_cmd::Command;
use assert_fs::fixture::NamedTempFile;
use assert_fs::prelude::*;
use predicates::prelude::*;

const PREFIX_CONFIG: &str = "match:\n  prefix: Arm\n  url: https://example.com?q=$1+$2\n";
const SUBMATCH_CONFIG: &str =
    "match:\n  prefix: animals/\n  match:\n    exact: bear\n    url: https://bears.org\n";

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
    run_apply(PREFIX_CONFIG, Some("Armadillo"), None)
        .success()
        .stdout(predicate::eq("https://example.com?q=Arm+adillo"));
}

#[test]
fn stdin_dash_match_outputs_redirect() {
    run_apply(PREFIX_CONFIG, Some("-"), Some("Armadillo"))
        .success()
        .stdout(predicate::eq("https://example.com?q=Arm+adillo"));
}

#[test]
fn stdin_implicit_match_outputs_redirect() {
    run_apply(PREFIX_CONFIG, None, Some("Armadillo"))
        .success()
        .stdout(predicate::eq("https://example.com?q=Arm+adillo"));
}

#[test]
fn no_match_exit_code_two() {
    run_apply(PREFIX_CONFIG, Some("Hello"), None)
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty());
}

#[test]
fn prefix_delegates_to_submatcher() {
    run_apply(SUBMATCH_CONFIG, Some("animals/bear"), None)
        .success()
        .stdout(predicate::eq("https://bears.org"));
}
