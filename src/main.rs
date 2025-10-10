use anyhow::Result;
use dlt_explorer::{Cli, Parser};

fn main() -> Result<()> {
    let args = Cli::parse();
    println!("{:?}", args);

    dlt_explorer::process_dlt(args)
}
