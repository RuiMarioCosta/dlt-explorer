mod cmd_line_parser;
mod dlt;

use anyhow::{Result, anyhow};
use dlt::Dlt;

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
    Ok(())
}

fn process_in_terminal(args: Cli) -> Result<()> {
    let Some(mut paths) = args.paths else {
        return Err(anyhow!("No DLT paths"));
    };

    if args.sort {
        paths.sort();
    }

    let dlt = Dlt::new(paths, args.filter);
    println!("{:?}", dlt);
    println!("{:?}", dlt.paths());
    println!("{:?}", dlt.filter());

    Ok(())
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
        };

        let result = process_dlt(args);

        assert!(result.is_err());
    }

    #[test]
    fn process_dlt_with_one_path() {
        let args = Cli {
            paths: Some(vec![PathBuf::from("path/to/file")]),
            filter: None,
            terminal: true,
            sort: true,
        };

        let result = process_dlt(args);

        assert!(result.is_ok());
    }
}
