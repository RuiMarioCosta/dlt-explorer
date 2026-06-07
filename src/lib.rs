mod cmd_line_parser;
mod desktop;
pub mod dlt;

use crate::dlt::payload::{MESSAGE_TYPE, decode_message_type_info};
use anyhow::{Result, anyhow};

pub use cmd_line_parser::{Cli, Parser};

pub fn process_dlt(args: Cli) -> Result<()> {
    if !args.terminal {
        println!("Entering Gui");
        process_in_gui(args)
    } else {
        println!("Using terminal");
        process_in_terminal(args)
    }
}

fn process_in_gui(_args: Cli) -> Result<()> {
    desktop::run_desktop_shell()?;

    Ok(())
}

fn process_in_terminal(args: Cli) -> Result<()> {
    let Some(mut paths) = args.paths else {
        return Err(anyhow!("No DLT paths"));
    };

    if args.sort {
        paths.sort();
    }

    let version = dlt::detect_version(&paths[0])?;
    for path in &paths[1..] {
        let v = dlt::detect_version(path)?;
        if v != version {
            return Err(anyhow!(
                "Mixed DLT versions: first file is v{} but {:?} is v{}",
                version,
                path,
                v
            ));
        }
    }

    println!("DLT Version: {}", version);

    if version == 1 {
        let (dlt, errors) = dlt::v1::Dlt::open(paths)?;
        if !errors.is_empty() {
            eprintln!("{} parse error(s) encountered", errors.len());
        }
        print_terminal_rows_v1(&dlt, args.limit);
    } else {
        let (dlt, errors) = dlt::v2::Dlt::open(paths)?;
        if !errors.is_empty() {
            eprintln!("{} parse error(s) encountered", errors.len());
        }
        print_terminal_rows_v2(&dlt, args.limit);
    }

    Ok(())
}

fn print_terminal_rows_v1(dlt: &dlt::v1::Dlt, limit: Option<usize>) {
    println!("idx\ttype\ttype_info\tecu\tapid\tctid\tpayload");
    let total = dlt.len();
    let rows_to_print = limit.unwrap_or(total).min(total);
    for i in 0..rows_to_print {
        let mstp = dlt.message_type(i) as usize;
        let mtin = dlt.message_type_info(i) as usize;
        let msg_type = MESSAGE_TYPE.get(mstp).copied().unwrap_or("");
        let type_info = decode_message_type_info(mstp, mtin);
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            i,
            msg_type,
            type_info,
            dlt.ecu(i),
            dlt.apid(i),
            dlt.ctid(i),
            dlt.payload_text(i)
        );
    }
    if rows_to_print < total {
        println!(
            "... truncated: showing {} of {} rows (use --limit to adjust)",
            rows_to_print, total
        );
    }
}

fn print_terminal_rows_v2(dlt: &dlt::v2::Dlt, limit: Option<usize>) {
    println!("idx\ttype\ttype_info\tecu\tapid\tctid\tpayload");
    let total = dlt.len();
    let rows_to_print = limit.unwrap_or(total).min(total);
    for i in 0..rows_to_print {
        let mstp = dlt.message_type(i) as usize;
        let mtin = dlt.message_type_info(i) as usize;
        let msg_type = MESSAGE_TYPE.get(mstp).copied().unwrap_or("");
        let type_info = decode_message_type_info(mstp, mtin);
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            i,
            msg_type,
            type_info,
            dlt.ecu(i),
            dlt.apid(i),
            dlt.ctid(i),
            dlt.payload_text(i)
        );
    }
    if rows_to_print < total {
        println!(
            "... truncated: showing {} of {} rows (use --limit to adjust)",
            rows_to_print, total
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn process_dlt_terminal_with_no_paths() {
        let args = Cli {
            paths: None,
            filter: None,
            terminal: true,
            sort: true,
            limit: None,
        };

        let result = process_dlt(args);

        assert!(result.is_err());
    }

    #[test]
    fn process_dlt_with_one_path() {
        let args = Cli {
            paths: Some(vec![PathBuf::from(
                env!("CARGO_MANIFEST_DIR").to_string()
                    + "/tests/data/testfile_control_messages.dlt",
            )]),
            filter: None,
            terminal: true,
            sort: true,
            limit: None,
        };

        let result = process_dlt(args);

        assert!(result.is_ok());
    }
}
