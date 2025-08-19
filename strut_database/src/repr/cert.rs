use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use sqlx_core::net::tls::CertificateInput;
use std::fmt::Formatter;
use std::path::PathBuf;

/// Closely replicates the `sqlx` crate’s [`CertificateInput`] enum, providing
/// the [deserialization](serde::de::Deserialize) capability via the original’s
/// implementation of `From<String>`.
#[derive(Debug)]
pub(crate) enum ProxyCertificateInput {
    /// The equivalent of [`CertificateInput::Inline`].
    Inline(Vec<u8>),

    /// The equivalent of [`CertificateInput::File`].
    File(PathBuf),
}

/// General trait implementations.
const _: () = {
    impl From<ProxyCertificateInput> for CertificateInput {
        fn from(value: ProxyCertificateInput) -> Self {
            match value {
                ProxyCertificateInput::Inline(bytes) => Self::Inline(bytes),
                ProxyCertificateInput::File(path) => Self::File(path),
            }
        }
    }

    impl From<CertificateInput> for ProxyCertificateInput {
        fn from(value: CertificateInput) -> Self {
            match value {
                CertificateInput::Inline(bytes) => ProxyCertificateInput::Inline(bytes),
                CertificateInput::File(path) => ProxyCertificateInput::File(path),
            }
        }
    }

    impl From<String> for ProxyCertificateInput {
        fn from(value: String) -> Self {
            CertificateInput::from(value).into()
        }
    }
};

/// Deserialize implementation.
const _: () = {
    impl<'de> Deserialize<'de> for ProxyCertificateInput {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_string(ProxyCertificateInputVisitor)
        }
    }

    struct ProxyCertificateInputVisitor;

    impl<'de> Visitor<'de> for ProxyCertificateInputVisitor {
        type Value = ProxyCertificateInput;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str(
                "a string with either an inline PEM-encoded certificate, or a path to a file",
            )
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(ProxyCertificateInput::from(value.to_string()))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(ProxyCertificateInput::from(value))
        }
    }
};
