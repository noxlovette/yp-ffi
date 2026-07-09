use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("failed to decode or encode image: {0}")]
    #[diagnostic(
        code(image_processor::image),
        help("check that the input path points to a valid, supported image format")
    )]
    Image(#[from] image::ImageError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Plugin(#[from] plugin_interface::Error),

    #[error("io error: {0}")]
    #[diagnostic(code(image_processor::io))]
    Io(#[from] std::io::Error),
}
