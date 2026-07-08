use clap::Parser;
use clio::*;
use plugin_interface::Plugin;
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
    #[arg(long, default_value = "-")]
    params: Input,
    #[arg(long, default_value = "target/debug")]
    plugin_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let mut args = Cli::parse();

    let img = image::open(args.input.path().path())?.to_rgba8();
    let (width, height) = img.dimensions();
    let mut data = img.into_raw();

    let mut params = String::new();
    args.params.read_to_string(&mut params)?;

    image_processor::plugin_loader::call_dynamic(
        &args.plugin_path,
        args.plugin,
        width,
        height,
        &mut data,
        &params,
    )?;

    image::save_buffer(
        args.output.path().path(),
        &data,
        width,
        height,
        image::ColorType::Rgba8,
    )?;

    Ok(())
}
