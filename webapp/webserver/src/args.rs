use clap::Parser;

/// Backend Worker
#[derive(Parser, Debug)]
pub struct Args {
    /// Configuration file
    #[clap(short, long, value_parser)]
    pub config: String,
}
