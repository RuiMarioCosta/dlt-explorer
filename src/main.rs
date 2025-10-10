use anyhow::Result;
use clap::Parser;

mod cmd_line_parser;

use cmd_line_parser::Cli;

fn main() -> Result<()> {
    let args = Cli::parse();
    println!("{:?}", args);

    if !args.terminal {
        println!("Entering Gui");
    } else {
        println!("Using terminal");
    }

    Ok(())
}
