use std::{borrow, panic};

use axum::http;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{IntoErrorChain, SourceKind, ToStatusCode, WithStatus};

/// Canonical error body shared by all our microservices (and expected by the limgrave
/// gateway): a stable `message_code` for programmatic handling, human-safe `details`,
/// and optional machine-readable `meta` (e.g. a list of conflicting appointments).
///
/// The underlying error that caused the failure is NOT serialized — it is logged via
/// `tracing` when the failure is built, so internal details never reach clients.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Failure {
    /// Stable machine-readable error code, e.g. `bad_request` or `appointment_conflict`.
    pub message_code: borrow::Cow<'static, str>,

    /// Human-safe description of the failure.
    pub details: borrow::Cow<'static, str>,

    /// Optional machine-readable payload accompanying the error.
    #[schema(value_type = Option<Object>)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl Failure {
    /// Creates related [`Builder`] instance.
    pub const fn builder() -> Builder {
        Builder {
            message: None,
            code: None,
            meta: None,
        }
    }
}

impl cubob::EmbedStruct for Failure {
    fn embed(&self, show: &mut cubob::StructShow) {
        show.field(&"message_code", &self.message_code)
            .field(&"details", &self.details);
    }
}

/// Helps to create [`Failure`] instances.
pub struct Builder {
    message: Option<borrow::Cow<'static, str>>,
    code: Option<borrow::Cow<'static, str>>,
    meta: Option<serde_json::Value>,
}

impl Builder {
    /// Sets the human-safe `details` text of the [`Failure`] being created.
    pub fn with_message(self, message: impl Into<borrow::Cow<'static, str>>) -> Self {
        Self {
            message: Some(message.into()),
            ..self
        }
    }

    /// Sets an explicit `message_code` (otherwise derived from the HTTP status).
    pub fn with_code(self, code: impl Into<borrow::Cow<'static, str>>) -> Self {
        Self {
            code: Some(code.into()),
            ..self
        }
    }

    /// Attaches machine-readable `meta` payload to the [`Failure`] being created.
    pub fn with_meta(self, meta: serde_json::Value) -> Self {
        Self {
            meta: Some(meta),
            ..self
        }
    }

    /// Adds arbitrary [`http::StatusCode`] value to [`Failure`] instance being created.
    pub fn with_status(self, code: http::StatusCode) -> BuilderWithStatus {
        BuilderWithStatus {
            general: self,
            code,
        }
    }

    /// Builds [`Failure`] instance from given error value with previously specified params.
    /// Value, returned by [`fn@ToStatusCode::to_status_code`] for E is used as related
    /// [`http::StatusCode`]. The error itself is logged, not serialized.
    #[track_caller]
    pub fn build<E, S: SourceKind>(self, error: E) -> WithStatus<Failure>
    where
        E: ToStatusCode + 'static,
        for<'a> &'a E: IntoErrorChain<S>,
    {
        let status = error.to_status_code();
        self.build_inner(status, &error, panic::Location::caller())
    }

    /// Returns a mapper which can build a [`Failure`] instance.
    /// See [`fn@Builder::build`] -- it is called inside that mapper.
    #[track_caller]
    pub fn into_mapper<E, S: SourceKind>(self) -> impl FnOnce(E) -> WithStatus<Failure>
    where
        E: ToStatusCode + 'static,
        for<'a> &'a E: IntoErrorChain<S>,
    {
        let caller = panic::Location::caller();
        move |error| {
            let status = error.to_status_code();
            self.build_inner(status, &error, caller)
        }
    }

    fn build_inner<E, S>(
        self,
        status: http::StatusCode,
        error: E,
        location: &panic::Location,
    ) -> WithStatus<Failure>
    where
        S: SourceKind,
        E: IntoErrorChain<S>,
    {
        let Self {
            message,
            code,
            meta,
        } = self;
        let failure = Failure {
            message_code: code
                .unwrap_or_else(|| borrow::Cow::Borrowed(status_message_code_fallback(status))),
            details: message
                .unwrap_or_else(|| borrow::Cow::Borrowed(status_details_fallback(status))),
            meta,
        };
        let x = error.into_error_chain();
        let y = x.enumerate();
        let error_chain_display = cubob::InstantStruct::inherit(y);
        tracing::error!(
            status = %status,
            message_code = %failure.message_code,
            location = %location,
            "request failed with error: {error_chain_display}"
        );
        WithStatus::new(failure).with_status(status)
    }
}

/// Helps to create [`Failure`] instances with explicitly specified [`http::StatusCode`] values.
pub struct BuilderWithStatus {
    general: Builder,
    code: http::StatusCode,
}

impl BuilderWithStatus {
    /// Sets the human-safe `details` text of the [`Failure`] being created.
    pub fn with_message(mut self, message: impl Into<borrow::Cow<'static, str>>) -> Self {
        self.general.message = Some(message.into());
        self
    }

    /// Sets an explicit `message_code` (otherwise derived from the HTTP status).
    pub fn with_code(mut self, code: impl Into<borrow::Cow<'static, str>>) -> Self {
        self.general.code = Some(code.into());
        self
    }

    /// Attaches machine-readable `meta` payload to the [`Failure`] being created.
    pub fn with_meta(mut self, meta: serde_json::Value) -> Self {
        self.general.meta = Some(meta);
        self
    }

    /// Builds [`Failure`] instance from given error value with previously specified params.
    /// Previously explicitly specified [`http::StatusCode`] value is used as related.
    /// The error itself is logged, not serialized.
    #[track_caller]
    pub fn build<E, S: SourceKind>(self, error: E) -> WithStatus<Failure>
    where
        E: 'static,
        for<'a> &'a E: IntoErrorChain<S>,
    {
        let Self { general, code } = self;
        general.build_inner(code, &error, panic::Location::caller())
    }

    /// Returns a mapper which can build a [`Failure`] instance.
    /// See [`fn@BuilderWithStatus::build`] -- it is called inside that mapper.
    #[track_caller]
    pub fn into_mapper<E, S: SourceKind>(self) -> impl FnOnce(E) -> WithStatus<Failure>
    where
        E: 'static,
        for<'a> &'a E: IntoErrorChain<S>,
    {
        let caller = panic::Location::caller();
        move |error| {
            let Self { general, code } = self;
            general.build_inner(code, &error, caller)
        }
    }
}

/// Default message code, used when the caller did not provide explicit values.
fn status_message_code_fallback(status: http::StatusCode) -> &'static str {
    match status {
        http::StatusCode::BAD_REQUEST => "bad_request",
        http::StatusCode::UNAUTHORIZED => "unauthorized",
        http::StatusCode::FORBIDDEN => "forbidden",
        http::StatusCode::NOT_FOUND => "not_found",
        http::StatusCode::CONFLICT => "conflict",
        http::StatusCode::SERVICE_UNAVAILABLE => "service_unavailable",
        _ if status.is_server_error() => "internal_error",
        _ => "unspecified_error",
    }
}

/// Default details for a status, used when the caller did not provide explicit values.
fn status_details_fallback(status: http::StatusCode) -> &'static str {
    match status {
        http::StatusCode::BAD_REQUEST => "Invalid request",
        http::StatusCode::UNAUTHORIZED => "Authentication is required",
        http::StatusCode::FORBIDDEN => "Access denied",
        http::StatusCode::NOT_FOUND => "Resource not found",
        http::StatusCode::CONFLICT => "Request could not be completed",
        http::StatusCode::SERVICE_UNAVAILABLE => "Service is temporarily unavailable",
        status if status.is_server_error() => "Internal server error",
        _ => "Request failed",
    }
}
