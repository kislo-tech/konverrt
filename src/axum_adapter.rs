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
