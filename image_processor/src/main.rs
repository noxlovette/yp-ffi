use clap::Parser;
use clio::*;
use plugin_interface::Plugin;
use std::{io::Read, path::PathBuf};
use tracing::{info, instrument};
use tracing_subscriber::EnvFilter;

#[derive(Debug, clap::Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long, short, value_parser, default_value = "-")]
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

#[instrument]
fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();

    let mut args = Cli::parse();

    info!("opening image");
    let img = image::open(args.input.path().path())?.to_rgba8();
    let (width, height) = img.dimensions();
    let mut data = img.into_raw();

    let mut params = String::new();
    args.params.read_to_string(&mut params)?;

    info!("calling plugin");
    plugin_interface::call_dynamic(
        &args.plugin_path,
        args.plugin,
        width,
        height,
        &mut data,
        &params,
    )?;

    info!("saving image");
    image::save_buffer(
        args.output.path().path(),
        &data,
        width,
        height,
        image::ColorType::Rgba8,
    )?;

    Ok(())
}
