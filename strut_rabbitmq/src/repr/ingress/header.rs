use crate::util::Morph;
use lapin::types::AMQPValue;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

/// Represents the value assigned to a RabbitMQ header, specifically the binding
/// header. This enumeration is used for deserializing
/// [`Ingress`](crate::Ingress) definitions.
///
/// ## Important: integer size
///
/// It is not made abundantly clear, but when matching headers for the purposes
/// of routing messages, RabbitMQ supports integer header values **only up to
/// 32-bit size**. At the same time, RabbitMQ supports 64-bit integer header
/// values just fine when simply attached to the message. Yet, during the routing
/// process, the headers with 64-bit integer values are ignored.
///
/// Thus, `i32` and `u32` are chosen to underlie the integer variants of this
/// enumeration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Header {
    /// Represents the boolean header value.
    Boolean(bool),
    /// Represents the signed integer header value.
    Int(i32),
    /// Represents the unsigned integer header value.
    UInt(u32),
    /// Represents the string header value.
    String(String),
}

impl Serialize for Header {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Boolean(b) => serializer.serialize_bool(*b),
            Self::Int(i) => serializer.serialize_i32(*i),
            Self::UInt(u) => serializer.serialize_u32(*u),
            Self::String(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for Header {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(HeaderVisitor)
    }
}

struct HeaderVisitor;

impl<'de> Visitor<'de> for HeaderVisitor {
    type Value = Header;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a RabbitMQ header value: a boolean, an integer, or a string")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Header::Boolean(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Header::Int(value.try_into().map_err(E::custom)?))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Header::UInt(value.try_into().map_err(E::custom)?))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Header::String(value.to_string()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Header::String(value))
    }
}

impl Header {
    /// Reports whether this [`Header`] may be considered empty (which
    /// means an empty string). Note that numerical zero values are **not**
    /// empty, and neither is `false`.
    pub fn is_empty(&self) -> bool {
        match self {
            Header::String(s) => s.is_empty(),
            _ => false,
        }
    }
}

impl From<Header> for AMQPValue {
    fn from(value: Header) -> Self {
        match value {
            Header::Boolean(b) => AMQPValue::morph(b),
            Header::Int(i) => AMQPValue::morph(i),
            Header::UInt(u) => AMQPValue::morph(u),
            Header::String(s) => AMQPValue::morph(s),
        }
    }
}

impl From<bool> for Header {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i8> for Header {
    fn from(value: i8) -> Self {
        Self::Int(value as i32)
    }
}

impl From<i16> for Header {
    fn from(value: i16) -> Self {
        Self::Int(value as i32)
    }
}

impl From<i32> for Header {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<u8> for Header {
    fn from(value: u8) -> Self {
        Self::UInt(value as u32)
    }
}

impl From<u16> for Header {
    fn from(value: u16) -> Self {
        Self::UInt(value as u32)
    }
}

impl From<u32> for Header {
    fn from(value: u32) -> Self {
        Self::UInt(value)
    }
}

impl From<&str> for Header {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<String> for Header {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn serialization() {
        // Given
        let input = vec![
            Header::Boolean(true),
            Header::Int(-1),
            Header::UInt(0),
            Header::UInt(1),
            Header::String("hello".into()),
        ];
        let expected_result = "[true,-1,0,1,\"hello\"]";

        // When
        let actual_result = serde_json::to_string(&input).unwrap();

        // Then
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn deserialization() {
        // Given
        let input = "[true,-1,0,1,\"hello\"]";
        let expected_result = vec![
            AMQPValue::morph(true),
            AMQPValue::LongInt(-1), // specify ambiguous types
            AMQPValue::LongUInt(0),
            AMQPValue::LongUInt(1),
            AMQPValue::morph("hello"),
        ];

        // When
        let actual_result = serde_json::from_str::<Vec<Header>>(input)
            .unwrap()
            .into_iter()
            .map(AMQPValue::from)
            .collect::<Vec<_>>();

        // Then
        assert_eq!(expected_result, actual_result);
    }
}
