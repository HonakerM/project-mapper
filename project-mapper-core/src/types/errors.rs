use std::{error::Error, fmt::Display};

// Standard error for invalid runtime configs. Inherits from String to
// make it simpler. This is really just for typing
// ! TODO make this error smarter
#[derive(Debug)]
pub struct RuntimeConfigValidationError(String);

impl Display for RuntimeConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for RuntimeConfigValidationError {}
impl From<String> for RuntimeConfigValidationError {
    fn from(msg: String) -> Self {
        RuntimeConfigValidationError(msg)
    }
}
impl From<&str> for RuntimeConfigValidationError {
    fn from(msg: &str) -> Self {
        RuntimeConfigValidationError(msg.to_string())
    }
}
