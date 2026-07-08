use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to decode or encode image: {0}")]
    Image(#[from] image::ImageError),

    #[error(transparent)]
    Plugin(#[from] plugin_interface::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
