use assert_cmd::Command;
use assert_fs::fixture::NamedTempFile;
use assert_fs::prelude::*;
use predicates::prelude::*;

const REGEX_CONFIG: &str = "match:\n  regex: (\\w+)\\.txt$\n  url: https://file.drive/$1.txt\n";
const SUBMATCH_CONFIG: &str = "match:\n  regex: ^animals/(\\w{1,3})\\w*\\.(\\w+)\n  match-with: $1.$2\n  match:\n    exact: Bea.pdf\n    url: https://animals.example/$1\n";

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
    run_apply(REGEX_CONFIG, Some("Bear.txt"), None)
        .success()
        .stdout(predicate::eq("https://file.drive/Bear.txt"));
}

#[test]
fn stdin_dash_match_outputs_redirect() {
    run_apply(REGEX_CONFIG, Some("-"), Some("Bear.txt"))
        .success()
        .stdout(predicate::eq("https://file.drive/Bear.txt"));
}

#[test]
fn stdin_implicit_match_outputs_redirect() {
    run_apply(REGEX_CONFIG, None, Some("Bear.txt"))
        .success()
        .stdout(predicate::eq("https://file.drive/Bear.txt"));
}

#[test]
fn no_match_exit_code_two() {
    run_apply(REGEX_CONFIG, Some("World"), None)
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty());
}

#[test]
fn list_picks_first_match() {
    run_apply(SUBMATCH_CONFIG, Some("animals/Bears.pdf"), None)
        .success()
        .stdout(predicate::eq("https://animals.example/Bea.pdf"));
}
