use assert_cmd::Command;
use assert_fs::fixture::NamedTempFile;
use assert_fs::prelude::*;
use predicates::prelude::*;

const FUZZY_CONFIG: &str = "match:\n  fuzzy: Elephant\n  url: https://heavy.animal?q=$1\n";

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
fn within_tolerance_outputs_redirect() {
    run_apply(FUZZY_CONFIG, Some("Elefant"), None)
        .success()
        .stdout(predicate::eq("https://heavy.animal?q=Elefant"));
}

#[test]
fn no_match_exit_code_two() {
    run_apply(FUZZY_CONFIG, Some("Cat"), None)
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty());
}

#[test]
fn respect_tolerance_setting() {
    let cfg = "match:\n  fuzzy: Elephant\n  tolerance: 1\n  url: https://heavy.animal\n";
    run_apply(cfg, Some("Elefant"), None)
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty());
}
