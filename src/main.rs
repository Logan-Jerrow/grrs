use anyhow::Context;
use clap::Parser;
use indicatif::ProgressBar;
use log::info;
use std::io::{Read, Write};

mod cli {
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
}

// TODO: use exitcode crate
// TODO: use crossbeam-channel for ctrl-c interrupts
// TODO: maybe use proptest crate
fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let path = &cli.path;

    info!("Pattern: {}", &cli.pattern);
    if cli.pattern.is_empty() {
        log::error!("pattern is empty");
    }
    info!("Path: {}", path.display());
    if cli.path.exists() {
        log::error!("path does not exist: {}", path.display());
    }

    // TODO: once_cell the progress bar
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb.set_message(format!("Reading file: {}", path.display()));

    // TODO: extract into function
    let file = std::fs::File::open(path)
        .with_context(|| format!("could not read file: {}", path.display()))?;
    info!("File opened");
    let mut reader = std::io::BufReader::new(file);

    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .with_context(|| format!("error while reading file: {}", path.display()))?;

    pb.set_message(format!("Searching for {}", &cli.pattern));

    let stdout = std::io::stdout();
    let mut writer = std::io::BufWriter::new(stdout.lock());
    grrs::find_matches(&content, &cli.pattern, &mut writer)?;

    pb.finish_and_clear();
    info!("Flushing writer");
    writer.flush()?;

    Ok(())
}
