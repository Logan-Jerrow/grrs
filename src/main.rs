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

static SPINNER: Lazy<ProgressBar> = Lazy::new(|| {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb
});

// TODO: use exitcode crate
// TODO: use crossbeam-channel for ctrl-c interrupts
// TODO: maybe use proptest crate
fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    check_args(&cli)?;

    let path = &cli.path;
    let content = get_contents(path)?;

    search_file(&content, &cli.pattern)?;

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

fn get_contents<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<String> {
    info!(target: "file", "reading file...");

    let path = path.as_ref();
    SPINNER.set_message(format!("reading file: {}", path.display()));

    let file = std::fs::File::open(path)
        .with_context(|| format!("could not read file: {}", path.display()))?;
    trace!("file opened");

    let mut reader = std::io::BufReader::new(file);
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .with_context(|| format!("error while reading file: {}", path.display()))?;

    info!(target: "file", "read done");
    Ok(content)
}

fn search_file(content: &str, pattern: &str) -> anyhow::Result<()> {
    info!(target: "search", "searching for pattern...");
    SPINNER.set_message(format!("Searching for {}", pattern));

    let mut writer = std::io::BufWriter::new(std::io::stdout().lock());
    grrs::find_matches(content, pattern, &mut writer)?;

    SPINNER.finish_and_clear();
    trace!("flushing writer");
    writer.flush()?;

    info!(target: "search", "search done");
    Ok(())
}
