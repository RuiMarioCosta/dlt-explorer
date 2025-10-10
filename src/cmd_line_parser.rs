pub use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to DLT files
    pub paths: Option<Vec<PathBuf>>,

    /// Path to DLT filter file
    #[arg(short, long)]
    pub filter: Option<PathBuf>,

    /// Use terminal as output
    #[arg(short, long, default_value_t = false)]
    pub terminal: bool,

    /// Sort DLT files by name
    #[arg(short, long, default_value_t = false)]
    pub sort: bool,
}
