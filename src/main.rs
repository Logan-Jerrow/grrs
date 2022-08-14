use anyhow::Context;
use clap::Parser;
use indicatif::ProgressBar;
use std::io::{Read, Write};

mod cli {
    use clap::Parser;
    use std::path::PathBuf;

    /// Search for a pattern in a file and display the lines that contain it.
    #[derive(Debug, Parser)]
    #[clap(author, version)]
    pub struct Cli {
        /// The pattern to look for
        pub pattern: String,

        /// The path to the file to read
        #[clap(value_parser)]
        pub path: PathBuf,
    }
}

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let path = &cli.path;

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    pb.set_message(format!("Reading file: {path:?}"));

    let file =
        std::fs::File::open(path).with_context(|| format!("could not read file: {:?}", path))?;
    let mut reader = std::io::BufReader::new(file);

    let mut contents = String::new();
    let bytes = reader
        .read_to_string(&mut contents)
        .with_context(|| format!("error while reading file: {:?}", path))?;

    pb.set_message(format!("Searching for {}", &cli.pattern));

    let stdout = std::io::stdout();
    let mut writer = std::io::BufWriter::new(stdout.lock());
    contents
        .lines()
        .filter(|line| line.contains(&cli.pattern))
        .try_for_each(|line| writeln!(writer, "{line}"))?;

    pb.finish_and_clear();
    writer.flush()?;

    println!("Read {bytes} bytes");
    Ok(())
}
