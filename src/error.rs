use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    Id3Error(id3::Error),
    FlacError(metaflac::Error),
    M4aError(mp4ameta::Error),
    IoError(std::io::Error),
    FmtError(String),
    DecoderError(base64::DecodeError),
    OtherError(symphonia::core::errors::Error),
    ImageError(imagesize::ImageError),
    NotSupportedError,
    UnknownError,
}
impl Default for Error {
    fn default() -> Self {
        Self::UnknownError
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FlacError(error) => Display::fmt(error, f),
            Error::Id3Error(error) => Display::fmt(error, f),
            Error::M4aError(error) => Display::fmt(error, f),
            Error::IoError(error) => Display::fmt(error, f),
            Error::FmtError(error) => Display::fmt(error, f),
            Error::DecoderError(error) => Display::fmt(error, f),
            Error::OtherError(err) => Display::fmt(err, f),
            Error::UnknownError => f.write_str("Unknown Error"),
            Error::ImageError(error) => Display::fmt(error, f),
            Error::NotSupportedError => f.write_str("Not Supported"),
        }
    }
}
impl std::error::Error for Error {}
impl From<id3::Error> for Error {
    fn from(value: id3::Error) -> Self {
        Self::Id3Error(value)
    }
}
impl From<metaflac::Error> for Error {
    fn from(value: metaflac::Error) -> Self {
        Self::FlacError(value)
    }
}

impl From<mp4ameta::Error> for Error {
    fn from(value: mp4ameta::Error) -> Self {
        Self::M4aError(value)
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
impl From<base64::DecodeError> for Error {
    fn from(value: base64::DecodeError) -> Self {
        Self::DecoderError(value)
    }
}

impl From<symphonia::core::errors::Error> for Error {
    fn from(value: symphonia::core::errors::Error) -> Self {
        use symphonia::core::errors::Error as OtherError;
        match value {
            OtherError::IoError(err) => Error::IoError(err),
            _ => Error::OtherError(value),
        }
    }
}
impl From<imagesize::ImageError> for Error {
    fn from(value: imagesize::ImageError) -> Self {
        Self::ImageError(value)
    }
}