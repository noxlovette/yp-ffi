use anstyle::{AnsiColor, Effects};
use clap::Parser;
use clio::*;
use image_processor::Error;
use indicatif::{ProgressBar, ProgressStyle};
use plugin_interface::Plugin;
use std::{io::Read, path::PathBuf, time::Duration};
use tracing::instrument;
use tracing_subscriber::EnvFilter;

fn styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Green.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Red.on_default())
}

#[derive(Debug, clap::Parser)]
#[command(version, about, styles = styles())]
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

fn spinner(msg: impl Into<std::borrow::Cow<'static, str>>) -> ProgressBar {
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(80));
    bar.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏✔"),
    );
    bar.set_message(msg);
    bar
}

#[instrument]
fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();

    let mut args = Cli::parse();

    let step = spinner("opening image");
    let img = image::open(args.input.path().path())
        .map_err(Error::Image)?
        .to_rgba8();
    let (width, height) = img.dimensions();
    let mut data = img.into_raw();
    step.finish_with_message("image opened");

    let mut params = String::new();
    args.params.read_to_string(&mut params).map_err(Error::Io)?;

    let step = spinner(format!("running {} plugin", args.plugin));
    plugin_interface::call_dynamic(
        &args.plugin_path,
        args.plugin,
        width,
        height,
        &mut data,
        &params,
    )
    .map_err(Error::Plugin)?;
    step.finish_with_message("plugin finished");

    let step = spinner("saving image");
    image::save_buffer(
        args.output.path().path(),
        &data,
        width,
        height,
        image::ColorType::Rgba8,
    )
    .map_err(Error::Image)?;
    step.finish_with_message("image saved");

    Ok(())
}
