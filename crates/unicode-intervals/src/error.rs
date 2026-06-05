use crate::constants::MAX_CODEPOINT;
use core::fmt;
use std::error;

/// Errors during Unicode intervals manipulations.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Provided category name is invalid.
    InvalidCategory(Box<str>),
    /// Provided Unicode version is invalid.
    InvalidVersion(Box<str>),
    /// Provided codepoints do not agree. Maximum should be greater or equal to minimum.
    InvalidCodepoints(u32, u32),
    /// Codepoint is not in the allowed range.
    CodepointNotInRange(u32, u32),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCategory(category) => f.write_fmt(format_args!(
                "'{category}' is not a valid Unicode category"
            )),
            Error::InvalidVersion(version) => {
                f.write_fmt(format_args!("'{version}' is not a valid Unicode version"))
            }
            Error::InvalidCodepoints(minimum, maximum) => f.write_fmt(format_args!(
                "Minimum codepoint should be less or equal than maximum codepoint. Got {minimum} < {maximum}"
            )),
            Error::CodepointNotInRange(minimum, maximum) => f.write_fmt(format_args!(
                "Codepoints should be in [0; {MAX_CODEPOINT}] range. Got: [{minimum}; {maximum}]"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_traits() {
        let error = Error::InvalidCodepoints(1, 1);
        assert_eq!(error, error);
        assert_eq!(format!("{error:?}"), "InvalidCodepoints(1, 1)");
    }
}
