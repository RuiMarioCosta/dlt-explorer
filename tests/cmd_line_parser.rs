use assert_cmd::prelude::*; // Add methods on commands
// use assert_fs::prelude::*;
// use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn default_subcommand_is_gui() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dlt-explorer")?;
    cmd.assert().success();

    Ok(())
}

#[test]
fn term_subcommand_requires_path() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dlt-explorer")?;
    cmd.arg("term").arg("path/to/file");
    cmd.assert().success();

    Ok(())
}
