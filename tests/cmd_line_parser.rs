use assert_cmd::prelude::*; // Add methods on commands
// use assert_fs::prelude::*;
// use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn call_without_options_or_paths_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dlt-explorer")?;
    cmd.assert().success();

    Ok(())
}

#[test]
fn call_with_one_path_suceeds() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dlt-explorer")?;
    cmd.arg("path/to/file");
    cmd.assert().success();

    Ok(())
}

#[test]
fn call_with_multiple_paths_suceeds() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dlt-explorer")?;
    cmd.arg("path/to/file1")
        .arg("path/to/file2")
        .arg("path/to/file3");
    cmd.assert().success();

    Ok(())
}

#[test]
fn check_existence_of_filter_option() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dlt-explorer")?;
    cmd.arg("-f").arg("path/to/filter");
    cmd.assert().success();

    Ok(())
}

#[test]
fn check_existence_of_terminal_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dlt-explorer")?;
    cmd.arg("-t");
    cmd.assert().success();

    Ok(())
}

#[test]
fn check_existence_of_sort_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dlt-explorer")?;
    cmd.arg("-s");
    cmd.assert().success();

    Ok(())
}

#[test]
fn call_with_multiple_parameters() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dlt-explorer")?;
    cmd.arg("path/to/file1")
        .arg("path/to/file2")
        .arg("-f")
        .arg("path/to/filter")
        .arg("-t");
    cmd.assert().success();

    Ok(())
}
