use anyhow::{anyhow, Context};
use clap::Parser;
use cli::Cli;
use grrserror::GrrsError;
use indicatif::ProgressBar;
use log::{debug, error, info, trace};
use once_cell::sync::Lazy;
use std::{
    io::{Read, Write},
    process::exit,
};

mod cli;
mod grrserror;
//
static SPINNER: Lazy<ProgressBar> = Lazy::new(|| {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb
});

// TODO: use crossbeam-channel for ctrl-c interrupts
// TODO: maybe use proptest crate
fn main() -> anyhow::Result<()> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("Error: {e}");
            exit(exitcode::USAGE)
        }
    };

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match run(cli) {
        Ok(_) => exit(exitcode::OK),
        Err(GrrsError { error, exit_code }) => {
            SPINNER.finish_and_clear();
            eprintln!("Error: {error:?}");
            exit(exit_code);
        }
    }
}

type Result<T> = anyhow::Result<T, GrrsError>;

fn run(cli: Cli) -> Result<()> {
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
fn check_args(cli: &cli::Cli) -> Result<()> {
    info!(target: "args", "validating args...");

    debug!(target: "pattern", "pattern: '{}'", &cli.pattern);
    debug!(target: "path", "path entered: '{}'", cli.path.display());

    // don't need to check if pattern or path is empty
    // ['clap(value_parser = NonEmptyStringValueParser::new()']

    match cli.path.try_exists() {
        Ok(true) => trace!(target: "path", "path exists"),
        Ok(false) => {
            let msg = "path does not exists";
            error!(target: "path", "{msg}");
            return Err(GrrsError::from_anyhow(anyhow!(msg), exitcode::NOINPUT));
        }
        Err(e) => {
            // i.e. lack of permissions
            let msg = format!("while checking existince of path: {e}");
            error!(target: "path", "{msg}");
            return Err(GrrsError::from_anyhow(anyhow!(msg), exitcode::NOINPUT));
        }
    }

    info!(target: "args", "validating args done");
    Ok(())
}

fn get_contents<P: AsRef<std::path::Path>>(path: P) -> Result<String> {
    info!(target: "file", "reading file...");

    let path = path.as_ref();
    SPINNER.set_message(format!("reading file: {}", path.display()));

    let file = std::fs::File::open(path)
        .with_context(|| format!("could not read file: {}", path.display()))
        .map_err(|error| GrrsError::from_anyhow(error, exitcode::NOINPUT))?;
    trace!("file opened");

    let mut reader = std::io::BufReader::new(file);
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .with_context(|| format!("while reading file: {}", path.display()))
        .map_err(|error| GrrsError::from_anyhow(error, exitcode::IOERR))?;

    info!(target: "file", "read done");
    Ok(content)
}

fn search_file(content: &str, pattern: &str) -> Result<()> {
    info!(target: "search", "searching for pattern...");
    SPINNER.set_message(format!("Searching for {}", pattern));

    let mut writer = std::io::BufWriter::new(std::io::stdout().lock());
    grrs::find_matches(content, pattern, &mut writer)
        .map_err(|error| GrrsError::from_anyhow(anyhow!(error), exitcode::IOERR))?;

    SPINNER.finish_and_clear();
    trace!("flushing writer");

    writer
        .flush()
        .map_err(|e| GrrsError::from_anyhow(anyhow!(e), exitcode::IOERR))?;

    info!(target: "search", "search done");
    Ok(())
}
