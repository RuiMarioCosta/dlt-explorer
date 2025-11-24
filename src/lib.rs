mod cmd_line_parser;
mod dlt;
mod gui;

use iced::Size;

use anyhow::{Result, anyhow};
use dlt::Dlt;
use gui::GUI;

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
    let _ = iced::application("DLT-Explorer", GUI::update, GUI::view)
        .subscription(GUI::subscription)
        .window_size(Size::new(1500.0, 600.0))
        .position(iced::window::Position::Centered)
        .resizable(true)
        .run();
    Ok(())
}

fn process_in_terminal(args: Cli) -> Result<()> {
    let Some(mut paths) = args.paths else {
        return Err(anyhow!("No DLT paths"));
    };

    if args.sort {
        paths.sort();
    }

    let dlt = Dlt::from_files(paths, args.filter);
    println!("{:?}", dlt);

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
            paths: Some(vec![PathBuf::from(
                env!("CARGO_MANIFEST_DIR").to_string()
                    + "/tests/data/testfile_control_messages.dlt",
            )]),
            filter: None,
            terminal: true,
            sort: true,
        };

        let result = process_dlt(args);

        assert!(result.is_ok());
    }
}
