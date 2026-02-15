use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI token usage analytics"));
}

#[test]
fn test_help_short_flag() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI token usage analytics"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokscale"));
}

#[test]
fn test_models_command_help() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("models")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show model usage report"));
}

#[test]
fn test_monthly_command_help() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("monthly")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show monthly usage report"));
}

#[test]
fn test_pricing_command_help() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("pricing")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show pricing for a model"));
}

#[test]
fn test_sources_command_help() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("sources")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show local scan locations"));
}

#[test]
fn test_graph_command_help() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("graph")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Export contribution graph data"));
}

#[test]
fn test_tui_command_help() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("tui")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Launch interactive TUI"));
}

#[test]
fn test_headless_command_help() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("headless")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Capture subprocess output"));
}

#[test]
fn test_login_command_help() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("login")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Login to Tokscale"));
}

#[test]
fn test_logout_command_help() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("logout")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Logout from Tokscale"));
}

#[test]
fn test_whoami_command_help() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("whoami")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show current logged in user"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("invalid-command").assert().failure();
}

#[test]
fn test_invalid_subcommand() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("models").arg("invalid-flag").assert().failure();
}

#[test]
fn test_pricing_command_missing_model() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("pricing").assert().failure();
}

#[test]
fn test_headless_command_missing_source() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("headless").assert().failure();
}

#[test]
fn test_headless_command_invalid_source() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("headless")
        .arg("invalid-source")
        .arg("test")
        .assert()
        .failure();
}

#[test]
fn test_models_with_invalid_date_format() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("models")
        .arg("--light")
        .arg("--since")
        .arg("invalid-date")
        .assert()
        .success();
}

#[test]
fn test_models_with_invalid_year() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("models")
        .arg("--light")
        .arg("--year")
        .arg("not-a-year")
        .assert()
        .success();
}

#[test]
fn test_global_theme_flag() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("--theme")
        .arg("blue")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_global_debug_flag() {
    let mut cmd = Command::cargo_bin("tokscale").unwrap();
    cmd.arg("--debug").arg("--help").assert().success();
}
