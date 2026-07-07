use clap::{Parser, ValueEnum};
use clio::*;
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(version, about)]
struct Cli {
    #[arg(value_parser, default_value = "-")]
    input: Input,
    #[arg(long, short, value_parser, default_value = "-")]
    output: Output,
    #[arg(short, long, value_enum)]
    plugin: Plugin,
    #[arg(short, long, default_value = "-")]
    params: Input,
    #[arg(short, long, default_value = "target/debug")]
    plugin_path: Option<PathBuf>,
}

/// The plugin to use. No prefixes, no extensions
#[derive(Clone, Debug, ValueEnum)]
enum Plugin {
    Mirror,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    Ok(())
}
