use std::fmt;

use axum::{http, response};
use serde::{Deserialize, Serialize};

use super::{make_json_response, status_code, ToStatusCode};

/// Represents an error explicitly bound with arbitrary [`http::StatusCode`] value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithStatus<T> {
    /// Value being bound with status code.
    value: T,

    /// Optional [`http::StatusCode`] value to bind with the value.
    /// When unspecified, default value ([`http::StatusCode::OK`]) is used.
    #[serde(default, with = "status_code")]
    code: http::StatusCode,
}

impl<T> WithStatus<T> {
    /// Creates new instance of [`WithStatus`] with given value and default [`http::StatusCode`].
    pub const fn new(value: T) -> Self {
        Self {
            value,
            code: http::StatusCode::OK,
        }
    }

    /// Returns instance of [`WithStatus`] with previously set value and new [`http::StatusCode`].
    pub const fn with_status(mut self, code: http::StatusCode) -> Self {
        self.code = code;
        self
    }

    /// Returns reference onto value contained within current [`WithStatus`] instance.
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Returns a mapper which constructs new instance of [`WithStatus`] from
    /// given value with code specified at the mapper creation moment.
    pub const fn map(code: http::StatusCode) -> impl FnOnce(T) -> Self {
        move |value| Self { value, code }
    }
}

impl<T> From<T> for WithStatus<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> ToStatusCode for WithStatus<T> {
    fn to_status_code(&self) -> http::StatusCode {
        self.code
    }
}

impl<T> fmt::Display for WithStatus<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        cubob::display_struct(f, &[(&"code", &self.code), (&"value", &self.value)])
    }
}

impl<T> response::IntoResponse for WithStatus<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> response::Response {
        let Self { value, code } = self;
        make_json_response(value, code)
    }
}
