use assert_cmd::prelude::*; // Add methods on commands
// use assert_fs::prelude::*;
// use predicates::prelude::*; // Used for writing assertions
use std::path::PathBuf;
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
    let path1 = PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_string() + "/tests/data/testfile_control_messages.dlt",
    );
    cmd.arg(path1);
    cmd.assert().success();

    Ok(())
}

#[test]
fn call_with_multiple_paths_suceeds() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dlt-explorer")?;
    let path1 = PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_string() + "/tests/data/testfile_control_messages.dlt",
    );
    let path2 = PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_string() + "/tests/data/testfile_single_payloads.dlt",
    );
    let path3 = PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_string() + "/tests/data/testfile_empty_number_and_text.dlt",
    );
    cmd.arg(path1).arg(path2).arg(path3);
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
    let path1 = PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_string() + "/tests/data/testfile_control_messages.dlt",
    );
    cmd.arg("-t").arg(path1);
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
    let path1 = PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_string() + "/tests/data/testfile_control_messages.dlt",
    );
    let path2 = PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_string() + "/tests/data/testfile_single_payloads.dlt",
    );
    cmd.arg(path1)
        .arg(path2)
        .arg("-f")
        .arg("path/to/filter")
        .arg("-t");
    cmd.assert().success();

    Ok(())
}
