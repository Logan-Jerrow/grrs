use anyhow::Context;
use clap::Parser;
use indicatif::ProgressBar;
use log::{debug, error, info, trace};
use once_cell::sync::Lazy;
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
    static SPINNER: Lazy<ProgressBar> = Lazy::new(|| {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(std::time::Duration::from_millis(120));
        pb
    });

    let cli = cli::Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    check_args(&cli)?;
    let path = &cli.path;

    SPINNER.set_message(format!("Reading file: {}", path.display()));

    // TODO: extract into function
    let file = std::fs::File::open(path)
        .with_context(|| format!("could not read file: {}", path.display()))?;
    info!("File opened");
    let mut reader = std::io::BufReader::new(file);

    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .with_context(|| format!("error while reading file: {}", path.display()))?;

    SPINNER.set_message(format!("Searching for {}", &cli.pattern));

    let stdout = std::io::stdout();
    let mut writer = std::io::BufWriter::new(stdout.lock());
    grrs::find_matches(&content, &cli.pattern, &mut writer)?;

    SPINNER.finish_and_clear();
    info!("Flushing writer");
    writer.flush()?;

    Ok(())
}

/// For logging argument values
///
/// While there might as well check too.
/// Clap should catch empty arguments before its called
fn check_args(cli: &cli::Cli) -> anyhow::Result<()> {
    info!(target: "args", "validating args...");
    debug!(target: "pattern", "pattern: '{}'", &cli.pattern);
    debug!(target: "path", "path entered: '{}'", cli.path.display());

    // don't need to check if pattern is empty
    // ['clap(value_parser = NonEmptyStringValueParser::new()']

    match cli.path.try_exists() {
        Ok(true) => trace!(target: "path", "path exists"),
        Ok(false) => {
            let e = "path does not exists";
            error!(target: "path", "{e}");
            anyhow::bail!(e);
        }
        Err(e) => {
            // i.e. lack of permissions
            let e = format!("error while checking existince of path: {e}");
            error!(target: "path", "{e}");
            anyhow::bail!(e);
        }
    }

    info!(target: "args", "validating args done");
    Ok(())
}
