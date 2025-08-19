use std::convert::Infallible;
use std::error::Error;
use std::string::FromUtf8Error;

/// Represents a way of decoding a payload of an incoming message (which is
/// received as a sequence of bytes) into an arbitrary result type.
///
/// It is important to know that both the original bytes (`Vec<u8>`) and the
/// decoded [`Result`](Decoder::Result) will be owned by the same
/// [`Envelope`](crate::Envelope). Given that Rust doesn’t allow
/// self-referential structs, we have to keep in mind that the result type may
/// not contain references to the original bytes.
///
/// Also, since the envelope is not easily destructed, only a reference to the
/// decoded payload is [exposed](crate::Envelope::payload).
///
/// For cases, where references to the original bytes are needed in the decoded
/// result, or where the decoded result must be owned by the external logic, the
/// byte slice may be [accessed](crate::Envelope::bytes) on the envelope for
/// manual decoding, and the provided [`NoopDecoder`] may be used as a dud.
pub trait Decoder {
    /// The type of decoded result.
    type Result;

    /// The type of error produced when decoding is not possible.
    type Error: Error;

    /// Decodes the given sequence of bytes into the desired
    /// [`Result`](Decoder::Result), or returns an appropriate
    /// [`Error`](Decoder::Error).
    fn decode(&self, bytes: &[u8]) -> Result<Self::Result, Self::Error>;
}

/// Implements [`Decoder`] for any function or closure that returns a
/// non-referential [`Result`].
///
/// If the result references the given `bytes`, this implementation will not
/// work. See the [`Decoder`] documentation for more details.
impl<F, R, E> Decoder for F
where
    F: Fn(&[u8]) -> Result<R, E>,
    E: Error,
{
    type Result = R;
    type Error = E;

    fn decode(&self, bytes: &[u8]) -> Result<Self::Result, Self::Error> {
        self(bytes)
    }
}

/// In some cases it is not necessary or not desirable to decode the incoming
/// message’s bytes on consumption. This convenience implementation of [`Decoder`]
/// enables such cases by not doing anything and returning a unit type `()`.
///
/// The original, un-decoded [`bytes`](crate::Envelope::bytes) of the
/// message are always available on the [`Envelope`](crate::Envelope).
///
/// See the [`Decoder`] documentation for more details.
pub struct NoopDecoder;

impl Decoder for NoopDecoder {
    type Result = ();
    type Error = Infallible;

    fn decode(&self, _bytes: &[u8]) -> Result<Self::Result, Self::Error> {
        Ok(())
    }
}

/// Implements [`Decoder`] that allocates an owned UTF-8 [`String`] with a copy
/// of the given bytes. This decoder fails with [`FromUtf8Error`] if the given
/// bytes cannot be interpreted as valid UTF-8.
#[derive(Default)]
pub struct StringDecoder;

impl Decoder for StringDecoder {
    type Result = String;
    type Error = FromUtf8Error;

    fn decode(&self, bytes: &[u8]) -> Result<Self::Result, Self::Error> {
        String::from_utf8(bytes.to_vec())
    }
}

/// This convenience implementation of [`Decoder`] enables automatically decoding
/// the incoming message’s bytes as a JSON into the given generic type `T`.
///
/// Since both the original bytes and the decoded object will be stored on the
/// same [`Envelope`](crate::Envelope), the decoded object may not reference the
/// original bytes. Otherwise, it would lead to a self-referential struct, which
/// Rust doesn’t allow.
///
/// This constraint is maintained by requiring `T` to be
/// [`DeserializeOwned`](serde::de::DeserializeOwned), which is automatically
/// implemented by types that implement [`Deserialize`](serde::Deserialize) and
/// don’t contain references.
#[cfg(feature = "json")]
pub struct JsonDecoder<T>(std::marker::PhantomData<T>);

#[cfg(feature = "json")]
impl<T> Default for JsonDecoder<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

#[cfg(feature = "json")]
impl<T> Decoder for JsonDecoder<T>
where
    T: serde::de::DeserializeOwned,
{
    type Result = T;
    type Error = serde_json::Error;

    fn decode(&self, bytes: &[u8]) -> Result<Self::Result, Self::Error> {
        serde_json::from_slice(bytes)
    }
}
