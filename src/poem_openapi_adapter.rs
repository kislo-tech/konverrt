use poem::http::StatusCode;
use poem_openapi::{
    payload::Json,
    types::{ToJSON, Type},
    ApiResponse, Object,
};

use crate::{Failure, WithStatus};

/// OpenAPI-friendly error response body.
///
/// This mirrors [`Failure`] without forcing the core type to implement
/// Poem OpenAPI traits directly.
#[derive(Debug, Clone, PartialEq, Object)]
pub struct FailureBody {
    /// Stable machine-readable error code, e.g. `bad_request`.
    pub message_code: String,

    /// Human-safe description of the failure.
    pub details: String,

    /// Optional machine-readable payload accompanying the error.
    #[oai(skip_serializing_if_is_none)]
    pub meta: Option<serde_json::Value>,
}

impl From<Failure> for FailureBody {
    fn from(value: Failure) -> Self {
        Self {
            message_code: value.message_code.into_owned(),
            details: value.details.into_owned(),
            meta: value.meta,
        }
    }
}

/// Poem OpenAPI response wrapper for Konverrt handler results.
///
/// `status_range` lets handlers keep using dynamic `WithStatus<T>` values while
/// still producing useful OpenAPI response metadata.
#[derive(ApiResponse)]
pub enum KonverrtResponse<T: Type + ToJSON> {
    #[oai(status_range = "2XX")]
    Success(StatusCode, Json<T>),

    #[oai(status_range = "4XX")]
    ClientError(StatusCode, Json<FailureBody>),

    #[oai(status_range = "5XX")]
    ServerError(StatusCode, Json<FailureBody>),
}

impl<T> From<std::result::Result<WithStatus<T>, WithStatus<Failure>>> for KonverrtResponse<T>
where
    T: Type + ToJSON,
{
    fn from(result: std::result::Result<WithStatus<T>, WithStatus<Failure>>) -> Self {
        match result {
            Ok(value) => {
                let (value, code) = value.into_parts();
                Self::Success(to_poem_status(code), Json(value))
            }
            Err(failure) => {
                let (failure, code) = failure.into_parts();
                let code = to_poem_status(code);
                let failure = Json(FailureBody::from(failure));

                if code.is_client_error() {
                    Self::ClientError(code, failure)
                } else {
                    Self::ServerError(code, failure)
                }
            }
        }
    }
}

fn to_poem_status(code: http::StatusCode) -> StatusCode {
    StatusCode::from_u16(code.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

#[cfg(test)]
mod tests {
    use poem_openapi::types::ToJSON as _;
    use serde_json::json;
    use std::collections::HashMap;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Object)]
    struct Payload {
        message: String,
        count: i32,
    }

    #[test]
    fn converts_success_response() {
        let result = Ok(WithStatus::new(Payload {
            message: "created".to_owned(),
            count: 2,
        })
        .with_status(http::StatusCode::CREATED));

        let response = KonverrtResponse::from(result);

        match response {
            KonverrtResponse::Success(code, Json(body)) => {
                assert_eq!(code, StatusCode::CREATED);
                assert_eq!(
                    body.to_json(),
                    Some(json!({
                        "message": "created",
                        "count": 2,
                    }))
                );
            }
            _ => panic!("expected success response"),
        }
    }

    #[test]
    fn converts_serializable_openapi_map_success_response() {
        let result = Ok(WithStatus::new(HashMap::from([(
            "status".to_owned(),
            "accepted".to_owned(),
        )]))
        .with_status(http::StatusCode::ACCEPTED));

        let response = KonverrtResponse::from(result);

        match response {
            KonverrtResponse::Success(code, Json(body)) => {
                assert_eq!(code, StatusCode::ACCEPTED);
                assert_eq!(body.to_json(), Some(json!({ "status": "accepted" })));
            }
            _ => panic!("expected success response"),
        }
    }

    #[test]
    fn converts_client_error_response() {
        let failure = Failure {
            message_code: "bad_request".into(),
            details: "Invalid request".into(),
            meta: Some(json!({ "field": "email" })),
        };

        let response: KonverrtResponse<Payload> =
            Err(WithStatus::new(failure).with_status(http::StatusCode::BAD_REQUEST)).into();

        match response {
            KonverrtResponse::ClientError(code, Json(body)) => {
                assert_eq!(code, StatusCode::BAD_REQUEST);
                assert_eq!(
                    body.to_json(),
                    Some(json!({
                        "message_code": "bad_request",
                        "details": "Invalid request",
                        "meta": { "field": "email" },
                    }))
                );
            }
            _ => panic!("expected client error response"),
        }
    }

    #[test]
    fn converts_server_error_response() {
        let failure = Failure {
            message_code: "internal_error".into(),
            details: "Internal server error".into(),
            meta: None,
        };

        let response: KonverrtResponse<Payload> =
            Err(WithStatus::new(failure).with_status(http::StatusCode::INTERNAL_SERVER_ERROR))
                .into();

        match response {
            KonverrtResponse::ServerError(code, Json(body)) => {
                assert_eq!(code, StatusCode::INTERNAL_SERVER_ERROR);
                assert_eq!(
                    body.to_json(),
                    Some(json!({
                        "message_code": "internal_error",
                        "details": "Internal server error",
                    }))
                );
            }
            _ => panic!("expected server error response"),
        }
    }
}
