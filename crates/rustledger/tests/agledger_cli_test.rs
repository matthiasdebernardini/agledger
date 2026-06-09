//! Integration tests for the agent-native agledger binary.

mod common;

use std::process::{Command, Output};

use common::test_fixtures_dir;

fn parse_json_stdout(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("agledger stdout should be JSON")
}

#[test]
fn test_agledger_root_returns_json_envelope() {
    let output = Command::new(require_agledger!())
        .output()
        .expect("failed to run agledger");

    assert!(output.status.success(), "root command should succeed");
    assert!(output.stderr.is_empty(), "stderr should be empty");

    let json = parse_json_stdout(&output);
    assert_eq!(json["ok"], true);
    assert_eq!(json["schema_version"], "agledger.v1");
    assert!(json["result"]["commands"].is_array());
}

#[test]
fn test_agledger_check_wraps_json_diagnostics() {
    let path = test_fixtures_dir().join("valid-ledger.beancount");
    if !path.exists() {
        eprintln!("Skipping: valid-ledger.beancount not found");
        return;
    }

    let output = Command::new(require_agledger!())
        .args(["check", "--format", "json"])
        .arg(&path)
        .output()
        .expect("failed to run agledger check");

    assert!(
        output.status.success(),
        "valid file should pass check: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["ok"], true);
    assert_eq!(json["result"]["command"], "check");
    assert_eq!(json["result"]["exit_status"], 0);
    assert!(json["result"]["stdout"].as_str().is_some());
    assert!(json["result"]["data"]["diagnostics"].is_array());
}

#[test]
fn test_agledger_check_preserves_validation_failure_details() {
    let path = test_fixtures_dir().join("validation-errors.beancount");
    if !path.exists() {
        eprintln!("Skipping: validation-errors.beancount not found");
        return;
    }

    let output = Command::new(require_agledger!())
        .args(["check", "--format", "json"])
        .arg(&path)
        .output()
        .expect("failed to run agledger check");

    assert!(
        !output.status.success(),
        "invalid file should fail check: stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["ok"], true);
    assert_eq!(json["exit_code"], 1);
    assert_eq!(json["result"]["command"], "check");
    assert_eq!(json["result"]["exit_status"], 1);
    assert_eq!(json["result"]["data"]["error_count"], 3);

    let diagnostics = json["result"]["data"]["diagnostics"]
        .as_array()
        .expect("diagnostics should be an array");
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic["code"] == "E1001" && diagnostic["phase"] == "validate"
        })
    );
}

#[test]
fn test_agledger_query_runs_bql_against_fixture() {
    let path = test_fixtures_dir().join("query-test.beancount");
    if !path.exists() {
        eprintln!("Skipping: query-test.beancount not found");
        return;
    }

    let output = Command::new(require_agledger!())
        .arg("query")
        .arg(&path)
        .args(["SELECT", "account", "FROM", "postings", "LIMIT", "3"])
        .output()
        .expect("failed to run agledger query");

    assert!(
        output.status.success(),
        "query should succeed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["ok"], true);
    assert_eq!(json["result"]["command"], "query");
    assert_eq!(json["result"]["exit_status"], 0);

    let stdout = json["result"]["stdout"]
        .as_str()
        .expect("query stdout should be a string");
    assert!(stdout.contains("Assets:Cash"));
    assert!(stdout.contains("3 row(s)"));
}

#[test]
fn test_agledger_report_balances_runs_against_fixture() {
    let path = test_fixtures_dir().join("valid-ledger.beancount");
    if !path.exists() {
        eprintln!("Skipping: valid-ledger.beancount not found");
        return;
    }

    let output = Command::new(require_agledger!())
        .arg("report")
        .arg(&path)
        .arg("balances")
        .output()
        .expect("failed to run agledger report");

    assert!(
        output.status.success(),
        "report should succeed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["ok"], true);
    assert_eq!(json["result"]["command"], "report");
    assert_eq!(json["result"]["exit_status"], 0);

    let stdout = json["result"]["stdout"]
        .as_str()
        .expect("report stdout should be a string");
    assert!(stdout.contains("Account Balances"));
    assert!(stdout.contains("Assets:Bank:Checking"));
}

#[test]
fn test_agledger_short_check_alias_maps_to_check() {
    let path = test_fixtures_dir().join("valid-ledger.beancount");
    if !path.exists() {
        eprintln!("Skipping: valid-ledger.beancount not found");
        return;
    }

    let output = Command::new(require_agledger!())
        .args(["c", "--format", "json"])
        .arg(&path)
        .output()
        .expect("failed to run agledger c");

    assert!(
        output.status.success(),
        "check alias should succeed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["ok"], true);
    assert_eq!(json["result"]["command"], "check");
    assert_eq!(json["result"]["exit_status"], 0);
    assert!(json["result"]["data"]["diagnostics"].is_array());
}
