use clap::{builder::NonEmptyStringValueParser, Parser};
use std::path::PathBuf;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Debug, Parser)]
#[clap(author, version)]
pub struct Cli {
    /// The pattern to look for
    // #[clap(forbid_empty_values = true)]
    #[clap(value_parser = NonEmptyStringValueParser::new())]
    pub pattern: String,

    /// The path to the file to read
    #[clap(value_parser)]
    pub path: PathBuf,

    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}
