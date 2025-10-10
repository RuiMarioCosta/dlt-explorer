mod cmd_line_parser;

pub use cmd_line_parser::{Cli, Parser};

pub fn process_dlt(args: Cli) {
    println!("{:?}", args);
}
