use std::fmt;

use axum::http;
use paste::paste;
use serde::{de, ser};

pub(super) fn serialize<S>(code: &http::StatusCode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    serializer.serialize_u16(code.as_u16())
}

macro_rules! visit_int {
    ($($typename:ty),+) => {
        $(
            paste! {
                fn [<visit_ $typename>]<E>(self, value: $typename) -> Result<Self::Value, E>
                where
                    E: de::Error
                {
                    let value = convert_int_to_u16(value, stringify!($typename)).map_err(E::custom)?;
                    convert_u16_to_status_code(value).map_err(E::custom)
                }
            }
        )+
    };
}

pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<http::StatusCode, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct Visitor;

    impl<'a> de::Visitor<'a> for Visitor {
        type Value = http::StatusCode;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a status code")
        }

        visit_int!(i128);
        visit_int!(i64);
        visit_int!(i32);
        visit_int!(i16);
        visit_int!(i8);

        visit_int!(u128);
        visit_int!(u64);
        visit_int!(u32);

        fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            convert_u16_to_status_code(v).map_err(E::custom)
        }

        fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            convert_u16_to_status_code(v as u16).map_err(E::custom)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let value = parse_str_as_u16(v).map_err(E::custom)?;
            convert_u16_to_status_code(value).map_err(E::custom)
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            str::from_utf8(v)
                .map_err(E::custom)
                .and_then(|text| self.visit_str(text))
        }
    }

    deserializer.deserialize_any(Visitor)
}

fn convert_int_to_u16<T>(
    value: T,
    typename: &'static str,
) -> Result<u16, errors::U16FromInt<T, <T as TryInto<u16>>::Error>>
where
    T: Copy + TryInto<u16>,
{
    value.try_into().map_err(|source| errors::U16FromInt {
        typename,
        value,
        source,
    })
}

fn parse_str_as_u16(text: &str) -> Result<u16, errors::U16FromStr> {
    text.parse().map_err(|source| errors::U16FromStr {
        value: text.to_owned(),
        source,
    })
}

fn convert_u16_to_status_code(value: u16) -> Result<http::StatusCode, errors::StatusFromU16> {
    http::StatusCode::from_u16(value).map_err(|source| errors::StatusFromU16 { value, source })
}

pub mod errors {
    use std::str;

    use super::http::status;

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    #[error("failed converting {typename} value '{value}' into u16: {source}")]
    pub struct U16FromInt<S, E> {
        pub(super) typename: &'static str,
        pub(super) value: S,
        pub(super) source: E,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("failed parsing text '{value}' as u16: {source}")]
    pub struct U16FromStr {
        pub(super) value: String,
        pub(super) source: <u16 as str::FromStr>::Err,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("failed converting u16 value '{value}' into status code: {source}")]
    pub struct StatusFromU16 {
        pub(super) value: u16,
        pub(super) source: status::InvalidStatusCode,
    }
}
