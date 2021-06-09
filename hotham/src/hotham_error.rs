use openxr::sys::Result as OpenXRResult;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HothamError {
    #[error("There was a problem with an OpenXR operation")]
    OpenXRError(#[from] OpenXRResult),
    #[error("The list was empty")]
    EmptyListError,
    #[error("The version of Vulkan or OpenXR is not supported")]
    UnsupportedVersionError,
    #[error("The format provided is not supported for this operation")]
    InvalidFormatError,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}