use std::fmt;

use serde::{Deserialize, Serialize};

use super::{status_code, ToStatusCode};

/// Represents an error explicitly bound with arbitrary [`http::StatusCode`] value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error<E> {
    /// Value of wrapped error.
    error: E,

    /// Optional [`http::StatusCode`] value to bind with the error.
    #[serde(with = "status_code")]
    code: http::StatusCode,
}

impl<E> Error<E> {
    /// Creates new instance of [`Error<E>`] based on given value of [`http::StatusCode`]
    /// and instance of `S` which implements [`Into<E>`].
    pub fn new<S>(code: http::StatusCode, error: S) -> Self
    where
        S: Into<E>,
    {
        Self {
            error: error.into(),
            code,
        }
    }

    /// Returns a mapper which lets to construct new [`Error<E>`]
    /// value from an `error` passed later with `code` given here and now.
    pub fn map<S>(code: http::StatusCode) -> impl FnOnce(S) -> Self
    where
        S: Into<E>,
    {
        move |error| Self::new(code, error)
    }
}

impl<E> ToStatusCode for Error<E> {
    fn to_status_code(&self) -> http::StatusCode {
        self.code
    }
}

impl<E> fmt::Display for Error<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}
