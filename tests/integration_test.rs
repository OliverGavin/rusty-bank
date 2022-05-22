use std::io::Write;
use std::process::Command;

use assert_cmd::prelude::*;
use itertools::Itertools;
use predicates::prelude::*;

use tempfile::NamedTempFile;

#[test]
fn test_failure_when_no_args() {
    let mut cmd = Command::cargo_bin("rusty-bank").unwrap();
    cmd.assert()
        .stderr(predicate::str::starts_with("Error: Usage: "))
        .failure();
}

#[test]
fn test_failure_when_no_file() {
    let mut cmd = Command::cargo_bin("rusty-bank").unwrap();
    cmd.arg("does_not_exist.csv")
        .assert()
        .stderr(predicate::str::contains("Error: No such file or directory"))
        .failure();
}

fn assert_stdout_eq(input: &str, expected: &'static str) {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", input).unwrap();

    let mut cmd = Command::cargo_bin("rusty-bank").unwrap();

    let cmd = cmd.arg(file.path());
    cmd.assert().success();

    let buf = cmd.output().unwrap().stdout;
    let expected = expected
        .replace(' ', "")
        .split("\n")
        .sorted()
        .rev()
        .join("\n");
    let output = String::from_utf8_lossy(&buf)
        .split("\n")
        .sorted()
        .rev()
        .join("\n");
    assert_eq!(expected, output);
}

#[test]
fn test_success_when_empty() {
    let input = "type, client, tx, amount\n";
    let expected = "";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_deposit_does_change_available_and_total() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        deposit,        2,  2,     20\n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        10,    0,    10,  false\n\
             2,        20,    0,    20,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_deposit_and_withdrawal_does_change_available_and_total() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        deposit,        2,  2,     20\n\
        withdrawal,     1,  3,      5\n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,         5,    0,     5,  false\n\
             2,        20,    0,    20,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_withdrawal_when_insufficient_funds_does_not_change_available_and_total() {
    let input = "\
        type,      client, tx, amount\n\
        withdrawal,     1,  1,      5\n\
        deposit,        1,  2,     10\n\
        withdrawal,     2,  3,     20\n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        10,    0,    10,  false\n\
             2,         0,    0,     0,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_dispute_when_no_resolve_or_chargeback_does_change_available_and_held_funds() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        deposit,        2,  2,     20\n\
        deposit,        1,  3,      5\n\
        dispute,        1,  3,       \n\
        dispute,        2,  2,       \n\
        deposit,        1,  4,      1\n\
        deposit,        2,  5,      1\n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        11,    5,    16,  false\n\
             2,         1,   20,    21,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_dispute_when_withdrawal_does_not_change_available_and_held_funds() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        withdrawal,     1,  3,      5\n\
        dispute,        1,  3,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,         5,    0,     5,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_dispute_when_insufficient_funds_does_change_funds() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        withdrawal,     1,  3,     10\n\
        dispute,        1,  1,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,       -10,   10,     0,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_dispute_when_transaction_does_not_exist_does_not_change_available_and_held_funds() {
    let input = "\
        type,      client, tx, amount\n\
        dispute,        1,  1,       \n\
        deposit,        1,  2,     10\n\
        dispute,        1,  5,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        10,    0,    10,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_dispute_when_transaction_does_not_match_client_id_does_not_change_available_and_held_funds()
{
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        dispute,        2,  1,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        10,    0,    10,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_resolve_when_dispute_does_change_available_and_held_funds() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        deposit,        2,  2,     20\n\
        deposit,        1,  3,      5\n\
        dispute,        1,  1,       \n\
        dispute,        2,  2,       \n\
        resolve,        1,  1,       \n\
        resolve,        2,  2,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        15,    0,    15,  false\n\
             2,        20,    0,    20,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_resolve_when_transaction_does_not_exist_does_not_change_available_and_held_funds() {
    let input = "\
        type,      client, tx, amount\n\
        resolve,        1,  1,       \n\
        resolve,        2,  2,       \n\
        deposit,        2,  2,     20\n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             2,        20,    0,    20,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_resolve_when_transaction_does_not_match_client_id_does_not_change_available_and_held_funds()
{
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        dispute,        1,  1,       \n\
        resolve,        2,  1,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,         0,   10,    10,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_resolve_when_dispute_does_not_exist_does_not_change_available_and_held_funds() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        resolve,        1,  1,       \n\
        deposit,        2,  2,     20\n\
        resolve,        2,  2,       \n\
        dispute,        2,  2,       \n\
        resolve,        2,  2,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        10,    0,    10,  false\n\
             2,        20,    0,    20,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_duplicate_dispute_before_resolve_is_ignored() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        deposit,        1,  2,      5\n\
        dispute,        1,  1,       \n\
        dispute,        1,  1,       \n\
        resolve,        1,  1,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        15,    0,    15,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
fn test_duplicate_dispute_after_resolve_is_ignored() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        deposit,        1,  2,      5\n\
        dispute,        1,  1,       \n\
        resolve,        1,  1,       \n\
        dispute,        1,  1,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        15,    0,    15,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
#[should_panic]
fn test_chargeback_when_dispute_does_change_total_and_held_funds_and_freeze() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,     10\n\
        deposit,        1,  2,      5\n\
        dispute,        1,  2,       \n\
        chargeback,     1,  2,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        10,    0,    10,   true\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
#[should_panic]
fn test_chargeback_when_transaction_does_not_exist_does_not_change_total_and_held_funds() {
    let input = "\
        type,      client, tx, amount\n\
        chargeback,     1,  1,       \n\
        deposit,        1,  2,     10\n\
        chargeback,     1,  1,       \n\
        dispute,        1,  3,       \n\
        chargeback,     1,  3,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        10,    0,    10,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
#[should_panic]
fn test_chargeback_when_transaction_does_not_match_client_id_does_not_change_total_and_held_funds()
{
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  2,      5\n\
        dispute,        1,  2,       \n\
        chargeback,     2,  2,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,         0,    5,     5,  false\n\
             2,         0,    0,     0,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
#[should_panic]
fn test_chargeback_when_dispute_does_not_exist_does_not_change_total_and_held_funds() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  2,     10\n\
        chargeback,     1,  2,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        10,    0,    10,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
#[should_panic]
fn test_chargeback_when_insufficient_funds_does_change_total_funds_to_negative_and_decrease_held_funds_and_freeze(
) {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,      5\n\
        withdrawal,     1,  2,      2\n\
        dispute,        1,  1,       \n\
        chargeback,     1,  1,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        -2,    0,    -2,   true\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
#[should_panic]
fn test_duplicate_dispute_before_chargeback_does_decrease_funds_and_freeze() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,      5\n\
        dispute,        1,  1,       \n\
        dispute,        1,  1,       \n\
        chargeback,     1,  1,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,         0,    0,     0,   true\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
#[should_panic]
fn test_any_transaction_after_chargeback_is_ignored() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  0,     10\n\
        deposit,        1,  1,      5\n\
        dispute,        1,  1,       \n\
        chargeback,     1,  1,       \n\
        dispute,        1,  1,       \n\
        deposit,        1,  2,     50\n\
        deposit,        1,  3,     20\n\
        dispute,        1,  3,       \n\
        withdrawal,     1,  4,      3\n\
        dispute,        1,  2,       \n\
        resolve,        1,  2,       \n\
        chargeback,     1,  3,       \n\
        deposit,        2,  5,     80\n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,        10,    0,    10,   true\n\
             2,        80,    0,    80,  false\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
#[should_panic]
fn test_resolve_after_chargeback_is_ignored() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,      5\n\
        dispute,        1,  1,       \n\
        chargeback,     1,  1,       \n\
        resolve,        1,  1,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,         0,    0,     0,   true\n\
    ";
    assert_stdout_eq(input, expected);
}

#[test]
#[should_panic]
fn test_chargeback_after_resolve_is_ignored() {
    let input = "\
        type,      client, tx, amount\n\
        deposit,        1,  1,      5\n\
        dispute,        1,  1,       \n\
        resolve,        1,  1,       \n\
        chargeback,     1,  1,       \n\
    ";
    let expected = "\
        client, available, held, total, locked\n\
             1,         5,    0,     5,  false\n\
    ";
    assert_stdout_eq(input, expected);
}
