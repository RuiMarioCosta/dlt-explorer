use anyhow::Result;

use dlt_explorer::{Cli, Parser};

fn main() -> Result<()> {
    let args = Cli::parse();
    println!("{:?}", args);

    if !args.terminal {
        println!("Entering Gui");
    } else {
        println!("Using terminal");
        dlt_explorer::process_dlt(args);
    }

    Ok(())
}
