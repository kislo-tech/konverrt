use axum::response::{IntoResponse, Response};

use crate::WithStatus;

impl<T> IntoResponse for WithStatus<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response {
        let (value, code) = self.into_parts();
        (code, axum::Json(value)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get as route_get, Router};
    use axum_test::{TestResponse, TestServer};
    use serde::Serialize;
    use serde_json::json;
    use std::collections::HashMap;
    use std::future::IntoFuture;

    use crate::{Failure, Success};

    #[derive(Serialize)]
    struct Payload {
        message: &'static str,
        count: usize,
    }

    fn app() -> Router {
        Router::new()
            .route("/payload", route_get(payload))
            .route("/success", route_get(success))
            .route("/failure", route_get(failure))
            .route("/bad-request", route_get(bad_request))
            .route("/internal-error", route_get(internal_error))
    }

    async fn payload() -> WithStatus<Payload> {
        WithStatus::new(Payload {
            message: "created",
            count: 2,
        })
        .with_status(http::StatusCode::CREATED)
    }

    async fn success() -> WithStatus<Success> {
        Success::new("done").with_status(http::StatusCode::ACCEPTED)
    }

    async fn failure() -> WithStatus<Failure> {
        WithStatus::new(Failure {
            message_code: "bad_request".into(),
            details: "Invalid request".into(),
            meta: Some(json!({ "field": "email" })),
        })
        .with_status(http::StatusCode::BAD_REQUEST)
    }

    async fn bad_request() -> WithStatus<HashMap<&'static str, &'static str>> {
        WithStatus::new(status_body("bad_request")).with_status(http::StatusCode::BAD_REQUEST)
    }

    async fn internal_error() -> WithStatus<HashMap<&'static str, &'static str>> {
        WithStatus::new(status_body("internal_error"))
            .with_status(http::StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn status_body(status: &'static str) -> HashMap<&'static str, &'static str> {
        HashMap::from([("status", status)])
    }

    fn get(server: &TestServer, path: &str) -> TestResponse {
        futures::executor::block_on(server.get(path).into_future())
    }

    #[test]
    fn preserves_status_codes_through_axum_router() {
        let server = TestServer::new(app());

        get(&server, "/bad-request")
            .assert_status(http::StatusCode::BAD_REQUEST)
            .assert_json(&json!({ "status": "bad_request" }));

        get(&server, "/internal-error")
            .assert_status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .assert_json(&json!({ "status": "internal_error" }));
    }

    #[test]
    fn serializes_json_object_response() {
        let server = TestServer::new(app());
        let response = get(&server, "/payload");

        response
            .assert_status(http::StatusCode::CREATED)
            .assert_header(http::header::CONTENT_TYPE, "application/json")
            .assert_json(&json!({
                "message": "created",
                "count": 2,
            }));
    }

    #[test]
    fn serializes_success_response() {
        let server = TestServer::new(app());
        let response = get(&server, "/success");

        response
            .assert_status(http::StatusCode::ACCEPTED)
            .assert_json(&json!({ "message": "done" }));
    }

    #[test]
    fn serializes_failure_response_without_internal_error_details() {
        let server = TestServer::new(app());
        let response = get(&server, "/failure");

        response
            .assert_status(http::StatusCode::BAD_REQUEST)
            .assert_json(&json!({
                "message_code": "bad_request",
                "details": "Invalid request",
                "meta": { "field": "email" },
            }));
    }
}
