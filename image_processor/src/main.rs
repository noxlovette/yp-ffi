use clap::{Parser, ValueEnum};
use clio::*;
use std::{io::Read, path::PathBuf};

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

fn main() -> anyhow::Result<()> {
    let mut args = Cli::parse();

    let img = image::open(args.input.path().path())?.to_rgba8();

    let ((width, height), data) = (img.dimensions(), img.as_raw());

    let mut params = String::new();
    args.params.read_to_string(&mut params)?;

    Ok(())
}
