use anyhow::Result;
use clap::Parser;
mod cmd_line_parser;
use cmd_line_parser::{Cli, Commands};

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Some(Commands::Gui(args)) => println!("Gui: {:?}", args.path),
        Some(Commands::Term(args)) => println!("Term: {:?}", args.path),
        None => println!("Should start GUI"),
    }

    Ok(())
}
