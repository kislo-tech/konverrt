mod error;
mod error_chain;
mod failure;
mod status_code;
mod success;
mod with_status;

#[cfg(feature = "axum")]
mod axum_adapter;

#[cfg(feature = "poem-openapi")]
pub mod poem_openapi_adapter;

use self::error_chain::{IntoErrorChain, SourceKind};
pub use self::{
    error::Error,
    error_chain::{source, ErrorChain},
    failure::Failure,
    success::Success,
    with_status::WithStatus,
};

/// Represents handler result convertable into [`http::response::Response`],
/// when `T` implements [`response::IntoResponse`]
pub type Result<T> = std::result::Result<T, WithStatus<Failure>>;

/// Represents handler result convertable into [`http::response::Response`],
/// when `T` implements [`serde::Serialize`]
pub type StatusedResult<T> = std::result::Result<WithStatus<T>, WithStatus<Failure>>;

/// Represents handler result convertable into [`http::response::Response`],
/// when no special return value is needed except for the success confiramtion.
/// `Ok` variant value should be [`Success`].
pub type SimpleResponse = Result<WithStatus<Success>>;

/// Trait for any type which values can return bound HTTP Status Code value.
pub trait ToStatusCode {
    fn to_status_code(&self) -> http::StatusCode;
}

impl ToStatusCode for http::StatusCode {
    fn to_status_code(&self) -> http::StatusCode {
        *self
    }
}
