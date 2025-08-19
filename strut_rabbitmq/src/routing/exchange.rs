use crate::{
    ExchangeKind, EXCHANGE_AMQ_DIRECT, EXCHANGE_AMQ_FANOUT, EXCHANGE_AMQ_HEADERS,
    EXCHANGE_AMQ_MATCH, EXCHANGE_AMQ_TOPIC, EXCHANGE_DEFAULT,
};
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};
use strut_factory::impl_deserialize_field;
use thiserror::Error;

/// Defines a RabbitMQ exchange to be used in definitions related to RabbitMQ
/// routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exchange {
    /// The RabbitMQ built-in default exchange (the empty-name exchange `""`).
    Default,

    /// The RabbitMQ built-in `"amq.direct"` exchange.
    AmqDirect,

    /// The RabbitMQ built-in `"amq.fanout"` exchange.
    AmqFanout,

    /// The RabbitMQ built-in `"amq.headers"` exchange.
    AmqHeaders,

    /// The RabbitMQ built-in `"amq.match"` exchange.
    AmqMatch,

    /// The RabbitMQ built-in `"amq.topic"` exchange.
    AmqTopic,

    /// A custom, non-built-in RabbitMQ exchange.
    Custom(CustomExchange),
}

impl Exchange {
    /// Reports whether the exchange is one of the built-ins for this definition.
    pub const fn is_builtin(&self) -> bool {
        match self {
            Self::Default
            | Self::AmqDirect
            | Self::AmqFanout
            | Self::AmqHeaders
            | Self::AmqMatch
            | Self::AmqTopic => true,
            Self::Custom(_) => false,
        }
    }

    /// Reports whether the exchange is [`default`](Exchange::Default)
    /// for this definition.
    pub const fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }

    /// Reports whether the exchange is [`custom`](Exchange::Custom) for
    /// this definition.
    pub const fn is_custom(&self) -> bool {
        !self.is_builtin()
    }

    /// Reports the exchange name for this definition.
    pub fn name(&self) -> &str {
        match self {
            Self::Default => EXCHANGE_DEFAULT,
            Self::AmqDirect => EXCHANGE_AMQ_DIRECT,
            Self::AmqFanout => EXCHANGE_AMQ_FANOUT,
            Self::AmqHeaders => EXCHANGE_AMQ_HEADERS,
            Self::AmqMatch => EXCHANGE_AMQ_MATCH,
            Self::AmqTopic => EXCHANGE_AMQ_TOPIC,
            Exchange::Custom(custom_exchange) => custom_exchange.name(),
        }
    }

    /// Reports the exchange kind for this definition.
    pub fn kind(&self) -> ExchangeKind {
        match self {
            Self::Default => ExchangeKind::Direct,
            Self::AmqDirect => ExchangeKind::Direct,
            Self::AmqFanout => ExchangeKind::Fanout,
            Self::AmqHeaders => ExchangeKind::Headers,
            Self::AmqMatch => ExchangeKind::Headers,
            Self::AmqTopic => ExchangeKind::Topic,
            Self::Custom(custom_exchange) => custom_exchange.kind(),
        }
    }

    /// Reports whether the exchange is durable for this definition.
    pub fn durable(&self) -> bool {
        match self {
            Self::Default
            | Self::AmqDirect
            | Self::AmqFanout
            | Self::AmqHeaders
            | Self::AmqMatch
            | Self::AmqTopic => true, // all built-in exchanges are durable
            Self::Custom(custom_exchange) => custom_exchange.durable(),
        }
    }

    /// Reports whether the exchange is auto-deleted for this definition.
    pub fn auto_delete(&self) -> bool {
        match self {
            Self::Default
            | Self::AmqDirect
            | Self::AmqFanout
            | Self::AmqHeaders
            | Self::AmqMatch
            | Self::AmqTopic => false, // built-in exchanges are never auto-deleted
            Self::Custom(custom_exchange) => custom_exchange.auto_delete(),
        }
    }
}

impl Display for Exchange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Default for Exchange {
    fn default() -> Self {
        Self::Default
    }
}

impl Exchange {
    /// Creates a new [`ExchangeBuilder`].
    pub fn builder() -> ExchangeBuilder {
        ExchangeBuilder::new()
    }

    /// Creates an [`Exchange`] with the given name. This can fail if a reserved name
    /// is used (all names starting with `amq.*` are reserved in RabbitMQ).
    pub fn named(name: impl Into<String>) -> Result<Exchange, ExchangeError> {
        Self::builder().with_name(name).build()
    }

    /// Returns a **built-in** exchange variant with the given name. If the given
    /// name doesn’t match any known built-in RabbitMQ exchange — [`None`] is
    /// returned.
    pub fn try_builtin_named(name: impl AsRef<str>) -> Option<Self> {
        match name.as_ref() {
            EXCHANGE_DEFAULT => Some(Self::Default),
            EXCHANGE_AMQ_DIRECT => Some(Self::AmqDirect),
            EXCHANGE_AMQ_FANOUT => Some(Self::AmqFanout),
            EXCHANGE_AMQ_HEADERS => Some(Self::AmqHeaders),
            EXCHANGE_AMQ_MATCH => Some(Self::AmqMatch),
            EXCHANGE_AMQ_TOPIC => Some(Self::AmqTopic),
            _ => None,
        }
    }
}

/// Represents the various error states that may arise when a RabbitMQ exchange
/// definition becomes invalid.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ExchangeError {
    /// Indicates a reserved name for a custom exchange.
    #[error("invalid name for custom exchange: name '{0}' is reserved")]
    CustomNameIsReserved(String),

    /// Indicates a mismatched kind for a built-in exchange.
    #[error(
        "invalid configuration for built-in exchange '{exchange}' ({exchange:?}): expected '{}', found '{given_kind}'",
        .exchange.kind(),
    )]
    MismatchedKindForBuiltin {
        /// Built-in exchange
        exchange: Exchange,
        /// Given (mismatched) kind
        given_kind: ExchangeKind,
    },

    /// Indicates a mismatched `durable` flag for a built-in exchange.
    #[error(
        "invalid configuration for built-in exchange '{exchange}' ({exchange:?}): expected durable={:?}, found durable={given_durable:?}",
        .exchange.durable(),
    )]
    MismatchedDurableForBuiltin {
        /// Built-in exchange
        exchange: Exchange,
        /// Given (mismatched) `durable` flag
        given_durable: bool,
    },

    /// Indicates a mismatched `auto_delete` flag for a built-in exchange.
    #[error(
        "invalid configuration for built-in exchange '{exchange}' ({exchange:?}): expected auto_delete={:?}, found auto_delete={given_auto_delete:?}",
        .exchange.auto_delete(),
    )]
    MismatchedAutoDeleteForBuiltin {
        /// Built-in exchange
        exchange: Exchange,
        /// Given (mismatched) `auto_delete` flag
        given_auto_delete: bool,
    },
}

/// Represents the various error states that may arise when a RabbitMQ **custom**
/// exchange definition becomes invalid.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum CustomExchangeError {
    /// Indicates an invalid name for a custom exchange.
    #[error("invalid name for custom exchange: exchange '{0}' ({0:?}) is a built-in exchange")]
    NameIsBuiltin(Exchange),

    /// Indicates a reserved name for a custom exchange.
    #[error("invalid name for custom exchange: name '{0}' is reserved")]
    NameIsReserved(String),
}

/// Defines a [**custom**](Exchange::Custom) RabbitMQ exchange, as opposed
/// to any built-in variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomExchange {
    name: String,
    kind: ExchangeKind,
    durable: bool,
    auto_delete: bool,
}

impl CustomExchange {
    /// Creates a new [`CustomExchangeBuilder`].
    pub fn builder() -> CustomExchangeBuilder {
        CustomExchangeBuilder::new()
    }

    /// Creates a **custom** [`Exchange`] with the given name. This can fail if
    /// the given name happens to match a known built-in RabbitMQ exchange, or if
    /// a reserved name is used (all names starting with `amq.*` are reserved in
    /// RabbitMQ).
    pub fn named(name: impl Into<String>) -> Result<Exchange, CustomExchangeError> {
        Self::builder().with_name(name).build()
    }
}

impl From<CustomExchange> for Exchange {
    fn from(value: CustomExchange) -> Self {
        Exchange::Custom(value)
    }
}

impl CustomExchange {
    /// Reports the exchange name for this definition.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Reports the exchange kind for this definition.
    pub fn kind(&self) -> ExchangeKind {
        self.kind
    }

    /// Reports whether the exchange is durable for this definition.
    pub fn durable(&self) -> bool {
        self.durable
    }

    /// Reports whether the exchange is auto-deleted for this definition.
    pub fn auto_delete(&self) -> bool {
        self.auto_delete
    }
}

/// Builds an [`Exchange`] incrementally and validates it on the final stage.
#[derive(Debug)]
pub struct ExchangeBuilder {
    name: String,
    kind: ExchangeKind,
    durable: bool,
    auto_delete: bool,
}

impl ExchangeBuilder {
    /// Creates a new [`Exchange`] builder.
    pub fn new() -> Self {
        Self {
            name: "".into(),
            kind: CustomExchange::default_kind(),
            durable: CustomExchange::default_durable(),
            auto_delete: CustomExchange::default_auto_delete(),
        }
    }

    /// Recreates this [`Exchange`] builder with the given name.
    pub fn with_name(self, name: impl Into<String>) -> Self {
        let name = name.into();

        if let Some(builtin_exchange) = Exchange::try_builtin_named(&name) {
            return Self {
                name,
                kind: builtin_exchange.kind(),
                durable: builtin_exchange.durable(),
                auto_delete: builtin_exchange.auto_delete(),
            };
        }

        Self { name, ..self }
    }

    /// Recreates this [`Exchange`] builder with the given kind.
    pub fn with_kind(self, kind: ExchangeKind) -> Self {
        Self {
            kind: kind.into(),
            ..self
        }
    }

    /// Recreates this [`Exchange`] builder with the given `durable` flag.
    pub fn with_durable(self, durable: bool) -> Self {
        Self { durable, ..self }
    }

    /// Recreates this [`Exchange`] builder with the given `auto_delete` flag.
    pub fn with_auto_delete(self, auto_delete: bool) -> Self {
        Self {
            auto_delete,
            ..self
        }
    }

    /// Finalizes the builder, validates its state, and, assuming valid state,
    /// returns the [`Exchange`].
    pub fn build(self) -> Result<Exchange, ExchangeError> {
        self.validate()?;

        if let Some(builtin_exchange) = Exchange::try_builtin_named(&self.name) {
            return Ok(builtin_exchange);
        }

        Ok(Exchange::Custom(CustomExchange {
            name: self.name,
            kind: self.kind,
            durable: self.durable,
            auto_delete: self.auto_delete,
        }))
    }

    fn validate(&self) -> Result<(), ExchangeError> {
        if let Some(builtin_exchange) = Exchange::try_builtin_named(&self.name) {
            if self.kind != builtin_exchange.kind() {
                return Err(ExchangeError::MismatchedKindForBuiltin {
                    exchange: builtin_exchange,
                    given_kind: self.kind,
                });
            };
            if self.durable != builtin_exchange.durable() {
                return Err(ExchangeError::MismatchedDurableForBuiltin {
                    exchange: builtin_exchange,
                    given_durable: self.durable,
                });
            };
            if self.auto_delete != builtin_exchange.auto_delete() {
                return Err(ExchangeError::MismatchedAutoDeleteForBuiltin {
                    exchange: builtin_exchange,
                    given_auto_delete: self.auto_delete,
                });
            };
        } else {
            if self.name.starts_with("amq.") {
                return Err(ExchangeError::CustomNameIsReserved(self.name.to_string()));
            }
        }

        Ok(())
    }
}

/// Builds a [`CustomExchange`] incrementally and validates it on the final
/// stage.
#[derive(Debug)]
pub struct CustomExchangeBuilder {
    name: String,
    kind: ExchangeKind,
    durable: bool,
    auto_delete: bool,
}

impl CustomExchangeBuilder {
    /// Creates a new [`CustomExchange`] builder.
    pub fn new() -> Self {
        Self {
            name: "".into(),
            kind: CustomExchange::default_kind(),
            durable: CustomExchange::default_durable(),
            auto_delete: CustomExchange::default_auto_delete(),
        }
    }

    /// Recreates this [`Exchange`] builder with the given name.
    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }

    /// Recreates this [`Exchange`] builder with the given kind.
    pub fn with_kind(self, kind: ExchangeKind) -> Self {
        Self {
            kind: kind.into(),
            ..self
        }
    }

    /// Recreates this [`Exchange`] builder with the given `durable` flag.
    pub fn with_durable(self, durable: bool) -> Self {
        Self { durable, ..self }
    }

    /// Recreates this [`Exchange`] builder with the given `auto_delete` flag.
    pub fn with_auto_delete(self, auto_delete: bool) -> Self {
        Self {
            auto_delete,
            ..self
        }
    }

    /// Finalizes the builder, validates its state, and, assuming valid state,
    /// returns the custom [`Exchange`].
    pub fn build(self) -> Result<Exchange, CustomExchangeError> {
        self.validate()?;

        Ok(Exchange::Custom(CustomExchange {
            name: self.name,
            kind: self.kind,
            durable: self.durable,
            auto_delete: self.auto_delete,
        }))
    }

    fn validate(&self) -> Result<(), CustomExchangeError> {
        if let Some(builtin_exchange) = Exchange::try_builtin_named(&self.name) {
            return Err(CustomExchangeError::NameIsBuiltin(builtin_exchange));
        } else {
            if self.name.starts_with("amq.") {
                return Err(CustomExchangeError::NameIsReserved(self.name.to_string()));
            }
        }

        Ok(())
    }
}

impl CustomExchange {
    fn default_kind() -> ExchangeKind {
        ExchangeKind::Direct
    }

    fn default_durable() -> bool {
        true
    }

    fn default_auto_delete() -> bool {
        false
    }
}

#[cfg(test)]
impl Default for CustomExchange {
    fn default() -> Self {
        Self {
            name: "".into(),
            kind: Self::default_kind(),
            durable: Self::default_durable(),
            auto_delete: Self::default_auto_delete(),
        }
    }
}

const _: () = {
    impl<'de> Deserialize<'de> for Exchange {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(ExchangeVisitor)
        }
    }

    struct ExchangeVisitor;

    impl<'de> Visitor<'de> for ExchangeVisitor {
        type Value = Exchange;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ exchange or a string exchange name")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Exchange::named(value).map_err(Error::custom)
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Exchange::named(value).map_err(Error::custom)
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut name: Option<String> = None;
            let mut kind = None;
            let mut durable = None;
            let mut auto_delete = None;

            while let Some(key) = map.next_key()? {
                match key {
                    ExchangeField::name => key.poll(&mut map, &mut name)?,
                    ExchangeField::kind => key.poll(&mut map, &mut kind)?,
                    ExchangeField::durable => key.poll(&mut map, &mut durable)?,
                    ExchangeField::auto_delete => key.poll(&mut map, &mut auto_delete)?,
                    ExchangeField::__ignore => map.next_value()?,
                };
            }

            let name = ExchangeField::name.take(name)?;
            let mut builder = Exchange::builder().with_name(name);

            if let Some(kind) = kind {
                builder = builder.with_kind(kind);
            }

            if let Some(durable) = durable {
                builder = builder.with_durable(durable);
            }

            if let Some(auto_delete) = auto_delete {
                builder = builder.with_auto_delete(auto_delete);
            }

            Ok(builder.build().map_err(Error::custom)?)
        }
    }

    impl_deserialize_field!(
        ExchangeField,
        strut_deserialize::Slug::eq_as_slugs,
        name,
        kind,
        durable,
        auto_delete,
    );
};

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn deserialize_from_empty() {
        // Given
        let input = "";

        // When
        let actual_output = serde_yml::from_str::<Exchange>(input);

        // Then
        assert!(actual_output.is_err());
    }

    #[test]
    fn deserialize_from_string() {
        // Given
        let input = "\"test_exchange\"";
        let expected_output = Exchange::Custom(CustomExchange {
            name: "test_exchange".into(),
            ..Default::default()
        });

        // When
        let actual_output = serde_json::from_str::<Exchange>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_from_full() {
        // Given
        let input = r#"
extra_field: ignored
name: test_exchange
kind: hash_key
durable: false
auto_delete: true
"#;
        let expected_output = Exchange::Custom(CustomExchange {
            name: "test_exchange".into(),
            kind: ExchangeKind::HashKey,
            durable: false,
            auto_delete: true,
            ..Default::default()
        });

        // When
        let actual_output = serde_yml::from_str::<Exchange>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_builtin() {
        // Given
        let input = r#"
name: amq.topic
"#;
        let expected_output = Exchange::AmqTopic;

        // When
        let actual_output = serde_yml::from_str::<Exchange>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn name_is_reserved() {
        // Given
        let input = "amq.custom";
        let expected_output = ExchangeError::CustomNameIsReserved(input.into());

        // When
        let actual_output = Exchange::named(input).unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_name_is_reserved() {
        // Given
        let input = r#"
name: amq.custom
"#;
        let expected_output = ExchangeError::CustomNameIsReserved("amq.custom".into());

        // When
        let actual_output = serde_yml::from_str::<Exchange>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn mismatched_kind_for_builtin() {
        // Given
        let expected_output = ExchangeError::MismatchedKindForBuiltin {
            exchange: Exchange::AmqFanout,
            given_kind: ExchangeKind::Headers,
        };

        // When
        let actual_output = Exchange::builder()
            .with_name(Exchange::AmqFanout.name())
            .with_kind(ExchangeKind::Headers)
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_mismatched_kind_for_builtin() {
        // Given
        let input = r#"
name: amq.fanout
kind: headers
"#;
        let expected_output = ExchangeError::MismatchedKindForBuiltin {
            exchange: Exchange::AmqFanout,
            given_kind: ExchangeKind::Headers,
        };

        // When
        let actual_output = serde_yml::from_str::<Exchange>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn mismatched_durable_for_builtin() {
        // Given
        let expected_output = ExchangeError::MismatchedDurableForBuiltin {
            exchange: Exchange::AmqMatch,
            given_durable: false,
        };

        // When
        let actual_output = Exchange::builder()
            .with_name(Exchange::AmqMatch.name())
            .with_durable(false)
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_mismatched_durable_for_builtin() {
        // Given
        let input = r#"
name: amq.match
durable: false
"#;
        let expected_output = ExchangeError::MismatchedDurableForBuiltin {
            exchange: Exchange::AmqMatch,
            given_durable: false,
        };

        // When
        let actual_output = serde_yml::from_str::<Exchange>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn mismatched_auto_delete_for_builtin() {
        // Given
        let expected_output = ExchangeError::MismatchedAutoDeleteForBuiltin {
            exchange: Exchange::AmqMatch,
            given_auto_delete: true,
        };

        // When
        let actual_output = Exchange::builder()
            .with_name(Exchange::AmqMatch.name())
            .with_auto_delete(true)
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_mismatched_auto_delete_for_builtin() {
        // Given
        let input = r#"
name: amq.match
auto_delete: true
"#;
        let expected_output = ExchangeError::MismatchedAutoDeleteForBuiltin {
            exchange: Exchange::AmqMatch,
            given_auto_delete: true,
        };

        // When
        let actual_output = serde_yml::from_str::<Exchange>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn custom_name_is_builtin() {
        // Given
        let expected_output = CustomExchangeError::NameIsBuiltin(Exchange::AmqHeaders);

        // When
        let actual_output = CustomExchange::named(Exchange::AmqHeaders.name()).unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn custom_name_is_reserved() {
        // Given
        let expected_output = CustomExchangeError::NameIsReserved("amq.custom".to_string());

        // When
        let actual_output = CustomExchange::named("amq.custom").unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }
}
