use std::{borrow, fmt};

use serde::{Deserialize, Serialize};

use super::WithStatus;

/// Represents successfull result value of handler, converted into a `String` or `&'static str`.
/// Inteneded to be used when no other meaningful value should be returned.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Success {
    /// Message to pass to client.
    pub message: borrow::Cow<'static, str>,
}

impl Success {
    /// Creates new instance of [`Success`] from given message.
    pub fn new<T>(message: T) -> Self
    where
        T: Into<borrow::Cow<'static, str>>,
    {
        Self {
            message: message.into(),
        }
    }

    /// Wraps current [`Success`] into [`WithStatus`] and binds given [`http::StatusCode`] value.
    pub fn with_status(self, code: http::StatusCode) -> WithStatus<Self> {
        WithStatus::new(self).with_status(code)
    }
}

impl<T> From<T> for Success
where
    T: Into<borrow::Cow<'static, str>>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for Success {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        cubob::display_struct(f, &[(&"message", &self.message)])
    }
}
