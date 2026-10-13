# Konverrt

<img src="misc/konverrt-kisa.png" alt="Pixelated black cat inside an envelope"/>

Konverrt is a small helper crate for turning Rust handler `Result`s into predictable HTTP responses.

It gives you a few plain data types for the shape your services return: values with status codes,
simple success messages, and client-safe error bodies. Framework-specific response glue lives behind
feature flags.

## Quick Look

```rust
use http::StatusCode;
use konverrt::{StatusedResult, WithStatus};

#[derive(serde::Serialize)]
struct UserDto {
    id: String,
}

async fn create_user() -> StatusedResult<UserDto> {
    let res = UserDto {
        id: "user_1".to_owned(),
    };

    Ok(WithStatus::new(res).with_status(StatusCode::CREATED))
}
```

With the `axum` feature enabled, `WithStatus<T>` implements Axum's `IntoResponse` when `T` is
serializable, so it can be returned from handlers directly.

## Installation

Konverrt has no default features. Installing it bare gives you the framework-neutral core types:

```toml
konverrt = "0.2"
```

Enable the adapter you actually use:

```toml
konverrt = { version = "0.2", features = ["axum"] }
konverrt = { version = "0.2", features = ["axum", "utoipa"] }
```

## Feature Flags

| Feature | What it enables |
| --- | --- |
| `axum` | Axum response conversion for `WithStatus<T>` |
| `utoipa` | OpenAPI schema derives for `Success` and `Failure` |

With no features enabled, the crate still gives you the shared response types, status handling, and
failure builders. Framework adapters are opt-in, so pulling Konverrt into a non-Axum service stays
quiet.

The crate uses `http::StatusCode` in its core types. Framework adapters convert from there when they
need to.

## Main Types

| Type | What it is for |
| --- | --- |
| `WithStatus<T>` | Binds a response value to an HTTP status code |
| `Failure` | Client-safe error body with `message_code`, `details`, and optional `meta` |
| `Success` | Small `{ "message": ... }` success body |
| `Result<T>` | Alias for `std::result::Result<T, WithStatus<Failure>>` |
| `StatusedResult<T>` | Alias for `std::result::Result<WithStatus<T>, WithStatus<Failure>>` |
| `SimpleResponse` | Alias for handlers that only need to return a statused `Success` |

## Success Responses

```rust
use http::StatusCode;
use konverrt::Success;

async fn accept_job() -> konverrt::SimpleResponse {
    Ok(Success::new("accepted").with_status(StatusCode::ACCEPTED))
}
```

Response body:

```json
{
  "message": "accepted"
}
```

## Error Responses

`Failure` keeps internal error details out of the client response. The source error is logged when
the failure is built, while the response gets stable, safe fields.

```rust
use http::StatusCode;
use konverrt::{source, Failure};

let failure = Failure::builder()
    .with_status(StatusCode::BAD_REQUEST)
    .with_code("bad_request")
    .with_message("Invalid request")
    .with_meta(serde_json::json!({ "field": "email" }))
    .build::<_, source::Displayable>("email is not valid");
```

Response body:

```json
{
  "message_code": "bad_request",
  "details": "Invalid request",
  "meta": {
    "field": "email"
  }
}
```

## Framework Support

Axum is supported through the `axum` feature.

OpenAPI schema generation is supported through the `utoipa` feature, which derives `ToSchema` for
the shared `Success` and `Failure` response bodies.

Other HTTP frameworks will be added.

## License

Konverrt is published under the GPL-2.0 license. See [LICENSE](LICENSE) for the full text.

The project is authored by `kislo.tech`. The license may change in the future.

## Repository Conventions

✨ Nevermind those. We are not dushnilas ✨
