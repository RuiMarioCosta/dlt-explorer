use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Use graphical interface
    Gui(GuiArgs),
    /// Use terminal
    Term(TermArgs),
}

#[derive(Args)]
pub struct CommonArgs {
    /// Path to DLT filter file
    #[arg(short, long)]
    pub filter: Option<PathBuf>,

    /// Sort DLT files by name
    #[arg(short, long)]
    pub sort: Option<bool>,
}

#[derive(Args)]
pub struct GuiArgs {
    /// Path to DLT files
    pub path: Option<Vec<PathBuf>>,

    #[command(flatten)]
    cmd: CommonArgs,
}

#[derive(Args)]
pub struct TermArgs {
    /// Path to DLT files
    #[arg(required = true)]
    pub path: Vec<PathBuf>,

    #[command(flatten)]
    cmd: CommonArgs,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
