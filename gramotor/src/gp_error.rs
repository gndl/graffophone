use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct GpError {}

impl fmt::Display for GpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GpError is here!")
    }
}

impl Error for GpError {
    fn description(&self) -> &str {
        "I'm the gphero of errors"
    }
}
