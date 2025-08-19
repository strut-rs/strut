use config::ConfigError;
use std::fmt::{Debug, Display, Formatter};

/// A cloneable representation of a configuration error.
///
/// This enum exists because the upstream [`ConfigError`] from the `config` crate
/// is not [`Clone`]. Strut caches configuration results, including errors, to
/// avoid costly reparsing. To support this caching, a cloneable error type
/// is required.
///
/// This enum mirrors the structure of [`ConfigError`] and provides `From`
/// implementations to convert from it. Most variants and their fields directly
/// correspond to their upstream counterparts.
///
/// [`ConfigError`]: ConfigError
#[derive(Clone)]
#[non_exhaustive]
pub enum AppConfigError {
    /// The equivalent of [`ConfigError::Frozen`].
    Frozen,

    /// The equivalent of [`ConfigError::NotFound`].
    NotFound(String),

    /// The equivalent of [`ConfigError::PathParse`].
    PathParse {
        /// The stringified equivalent of the `cause` field on
        /// [`ConfigError::PathParse`].
        cause_message: String,
    },

    /// The equivalent of [`ConfigError::FileParse`].
    FileParse {
        /// The equivalent of the `uri` field on [`ConfigError::FileParse`].
        uri: Option<String>,

        /// The stringified equivalent of the `cause` field on
        /// [`ConfigError::FileParse`].
        cause_message: String,
    },

    /// The equivalent of [`ConfigError::Type`].
    Type {
        /// The equivalent of the `origin` field on [`ConfigError::Type`].
        origin: Option<String>,

        /// The stringified equivalent of the `unexpected` field on
        /// [`ConfigError::Type`].
        unexpected_content: String,

        /// The equivalent of the `expected` field on [`ConfigError::Type`].
        expected: &'static str,

        /// The equivalent of the `key` field on [`ConfigError::Type`].
        key: Option<String>,
    },

    /// The equivalent of [`ConfigError::At`].
    At {
        /// The stringified equivalent of the `error` field on
        /// [`ConfigError::At`].
        error_message: String,

        /// The equivalent of the `origin` field on [`ConfigError::At`].
        origin: Option<String>,

        /// The equivalent of the `key` field on [`ConfigError::At`].
        key: Option<String>,
    },

    /// The equivalent of [`ConfigError::Message`].
    Message(String),

    /// The stringified equivalent of [`ConfigError::Foreign`].
    Foreign(String),

    /// Covers any additional variants of [`ConfigError`] that may appear in the
    /// future (since the enum is marked with `#[non_exhaustive]`).
    Unsupported(String),
}

/// Enables consuming a [`ConfigError`] to make an owned [`AppConfigError`].
impl From<ConfigError> for AppConfigError {
    fn from(value: ConfigError) -> Self {
        match value {
            ConfigError::Frozen => Self::Frozen,
            ConfigError::NotFound(value) => Self::NotFound(value),
            ConfigError::PathParse { cause } => Self::PathParse {
                cause_message: cause.to_string(),
            },
            ConfigError::FileParse { uri, cause } => Self::FileParse {
                uri,
                cause_message: cause.to_string(),
            },
            ConfigError::Type {
                origin,
                unexpected,
                expected,
                key,
            } => Self::Type {
                origin,
                unexpected_content: unexpected.to_string(),
                expected,
                key,
            },
            ConfigError::At { error, origin, key } => Self::At {
                error_message: error.to_string(),
                origin,
                key,
            },
            ConfigError::Message(value) => Self::Message(value),
            ConfigError::Foreign(value) => Self::Foreign(value.to_string()),
            _ => Self::Unsupported(value.to_string()),
        }
    }
}

/// Enables transmuting a reference to [`ConfigError`] to an owned
/// [`AppConfigError`].
impl From<&ConfigError> for AppConfigError {
    fn from(value: &ConfigError) -> Self {
        match value {
            ConfigError::Frozen => Self::Frozen,
            ConfigError::NotFound(value) => Self::NotFound(value.clone()),
            ConfigError::PathParse { cause } => Self::PathParse {
                cause_message: cause.to_string(),
            },
            ConfigError::FileParse { uri, cause } => Self::FileParse {
                uri: uri.clone(),
                cause_message: cause.to_string(),
            },
            ConfigError::Type {
                origin,
                unexpected,
                expected,
                key,
            } => Self::Type {
                origin: origin.clone(),
                unexpected_content: unexpected.to_string(),
                expected,
                key: key.clone(),
            },
            ConfigError::At { error, origin, key } => Self::At {
                error_message: error.to_string(),
                origin: origin.clone(),
                key: key.clone(),
            },
            ConfigError::Message(value) => Self::Message(value.clone()),
            ConfigError::Foreign(value) => Self::Foreign(value.to_string()),
            _ => Self::Unsupported(value.to_string()),
        }
    }
}

/// Replicates the [`Debug`] implementation of [`ConfigError`] verbatim, for the
/// lack of a better option.
impl Debug for AppConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self)
    }
}

/// Replicates the [`Display`] implementation of [`ConfigError`] verbatim, for
/// the lack of a better option.
impl Display for AppConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Frozen => write!(f, "configuration is frozen"),

            Self::PathParse { ref cause_message } => write!(f, "{cause_message}"),

            Self::Message(ref value) => write!(f, "{value}"),

            Self::Foreign(ref value) => write!(f, "{value}"),

            Self::NotFound(ref value) => {
                write!(f, "configuration property {value} not found")
            }

            Self::Type {
                ref origin,
                ref unexpected_content,
                expected,
                ref key,
            } => {
                write!(f, "invalid type: {unexpected_content}, expected {expected}")?;

                if let Some(ref key) = *key {
                    write!(f, " for key `{key}`")?;
                }

                if let Some(ref origin) = *origin {
                    write!(f, " in {origin}")?;
                }

                Ok(())
            }

            Self::At {
                ref error_message,
                ref origin,
                ref key,
            } => {
                write!(f, "{error_message}")?;

                if let Some(ref key) = *key {
                    write!(f, " for key `{key}`")?;
                }

                if let Some(ref origin) = *origin {
                    write!(f, " in {origin}")?;
                }

                Ok(())
            }

            Self::FileParse {
                ref uri,
                ref cause_message,
            } => {
                write!(f, "{cause_message}")?;

                if let Some(ref uri) = *uri {
                    write!(f, " in {uri}")?;
                }

                Ok(())
            }

            AppConfigError::Unsupported(ref value) => write!(f, "{value}"),
        }
    }
}
