use thiserror::Error;

#[derive(Error, Debug)]
#[error("{message:}")]
pub struct NesError {
    pub message: String,
}

impl NesError {
    pub fn new(message: &str) -> Self {
        NesError {
            message: message.to_string(),
        }
    }
}
