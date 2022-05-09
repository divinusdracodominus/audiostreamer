use err_derive::Error;
use std::error::Error;

#[derive(Debug, Error)]
pub enum OutputErr {
    #[error(
        display = "device {} not found when trying to spawn speaker threads",
        _0
    )]
    DeviceNotFound(String),
}
