use anyhow::{anyhow, Context};
use clap::Parser;
use cli::Cli;
use grrserror::{GrrsError, GrrsResult, Wrap};
use indicatif::ProgressBar;
use log::{debug, info, trace};
use once_cell::sync::Lazy;
use std::io::{Read, Write};

mod cli;
mod grrserror;

// TODO: use crossbeam-channel for ctrl-c interrupts
// TODO: maybe use proptest crate
fn main() -> Wrap {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            env_logger::Builder::new().init();
            return Wrap(Err(GrrsError::from_anyhow(anyhow!(e), exitcode::USAGE)));
        }
    };

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    Wrap(run(cli))
}

static SPINNER: Lazy<ProgressBar> = Lazy::new(|| {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb
});

fn run(cli: Cli) -> GrrsResult<()> {
    check_args(&cli)?;

    SPINNER.set_message(format!("reading file: {}", cli.path.display()));
    let content = get_contents(cli.path)?;

    SPINNER.set_message(format!("Searching for {}", cli.pattern));
    search_file(&content, &cli.pattern)?;

    Ok(())
}

/// For logging argument values
///
/// While there might as well check too.
/// Clap should catch empty arguments before its called
fn check_args(cli: &cli::Cli) -> GrrsResult<()> {
    info!(target: "args", "validating args...");

    debug!(target: "pattern", "pattern: '{}'", &cli.pattern);
    debug!(target: "path", "path entered: '{}'", cli.path.display());

    // don't need to check if pattern or path is empty
    // ['clap(value_parser = NonEmptyStringValueParser::new()']

    match cli.path.try_exists() {
        Ok(true) => {
            trace!(target: "path", "path exists");
            info!(target: "args", "validating args done");
            Ok(())
        }
        Ok(false) => Err(GrrsError::from_anyhow(
            anyhow!("path does not exists"),
            exitcode::NOINPUT,
        ))?,
        // i.e. lack of permissions
        Err(e) => Err(GrrsError::from_anyhow(
            anyhow!(format!("while checking existince of path: {e}")),
            exitcode::NOINPUT,
        ))?,
    }
}

fn get_contents<P: AsRef<std::path::Path>>(path: P) -> GrrsResult<String> {
    info!(target: "file", "reading file...");

    let path = path.as_ref();

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

fn search_file(content: &str, pattern: &str) -> GrrsResult<()> {
    info!(target: "search", "searching for pattern...");

    let mut writer = std::io::BufWriter::new(std::io::stdout().lock());
    grrs::find_matches(content, pattern, &mut writer)
        .map_err(|error| GrrsError::from_anyhow(anyhow!(error), exitcode::IOERR))?;

    trace!("flushing writer");
    SPINNER.finish_and_clear(); // clear progress bar before flush

    writer
        .flush()
        .map_err(|e| GrrsError::from_anyhow(anyhow!(e), exitcode::IOERR))?;

    info!(target: "search", "search done");
    Ok(())
}
