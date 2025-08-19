use crate::{ConfirmationLevel, Exchange, ExchangeKind};
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::Arc;
use strut_deserialize::{Slug, SlugMap};
use strut_factory::impl_deserialize_field;
use thiserror::Error;

/// Represents a collection of uniquely named [`Egress`] definitions.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EgressLandscape {
    egresses: SlugMap<Egress>,
}

/// Defines an outbound path for messages being sent into a RabbitMQ cluster.
///
/// Note, that the exchange is just a string name and not a full-fledged
/// [`Exchange`] definition. This is because the exchange is only referenced
/// by name on the outbound side. The responsibility for declaring an exchange is
/// entirely on the inbound side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Egress {
    name: Arc<str>,
    exchange: Arc<str>,
    routing_key: Arc<str>,
    confirmation: ConfirmationLevel,
    force_durable: bool,
}

impl EgressLandscape {
    /// Reports whether this landscape contains a [`Egress`] with the
    /// given unique name.
    pub fn contains(&self, name: impl AsRef<str>) -> bool {
        self.egresses.contains_key(name.as_ref())
    }

    /// Retrieves `Some` reference to a [`Egress`] from this landscape
    /// under the given name, or `None`, if the name is not present in the
    /// landscape.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&Egress> {
        self.egresses.get(name.as_ref())
    }

    /// Retrieves a reference to a [`Egress`] from this landscape under
    /// the given name. Panics if the name is not present in the collection.
    pub fn expect(&self, name: impl AsRef<str>) -> &Egress {
        let name = name.as_ref();

        self.get(name)
            .unwrap_or_else(|| panic!("requested an undefined RabbitMQ egress '{}'", name))
    }
}

impl Egress {
    /// Creates a new [`EgressBuilder`].
    pub fn builder() -> EgressBuilder {
        EgressBuilder::new()
    }
}

impl Egress {
    /// Reports the egress name for this definition.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Reports the egress exchange name for this definition.
    pub fn exchange(&self) -> &str {
        &self.exchange
    }

    /// Reports the egress routing key for this definition.
    pub fn routing_key(&self) -> &str {
        &self.routing_key
    }

    /// Reports the egress confirmation level for this definition.
    pub fn confirmation(&self) -> ConfirmationLevel {
        self.confirmation
    }

    /// Reports the egress `force_durable` flag for this definition.
    pub fn force_durable(&self) -> bool {
        self.force_durable
    }
}

impl Egress {
    /// Reports whether this definition requires any sending confirmation beyond
    /// the bare minimum of network transmission. If so, this should prompt the
    /// publisher to enable publisher confirms on the RabbitMQ channel.
    pub(crate) fn requires_any_confirmation(&self) -> bool {
        match self.confirmation {
            ConfirmationLevel::Transmitted => false,
            ConfirmationLevel::Accepted => true,
            ConfirmationLevel::Routed => true,
        }
    }

    /// Reports whether this definition warrants a `mandatory` flag on the
    /// RabbitMQ `basic_publish` call.
    pub(crate) fn requires_mandatory_publish(&self) -> bool {
        match self.confirmation {
            ConfirmationLevel::Transmitted => false,
            ConfirmationLevel::Accepted => false,
            ConfirmationLevel::Routed => true,
        }
    }
}

/// Builds an [`Egress`] incrementally and validates it on the final stage.
#[derive(Debug)]
pub struct EgressBuilder {
    name: Arc<str>,
    exchange: Arc<str>,
    routing_key: Arc<str>,
    confirmation: ConfirmationLevel,
    force_durable: bool,
}

impl EgressBuilder {
    /// Creates a new [`Egress`] builder.
    pub fn new() -> Self {
        Self {
            name: Arc::from(Egress::default_name()),
            exchange: Arc::from(Egress::default_exchange()),
            routing_key: Arc::from(Egress::default_routing_key()),
            confirmation: Egress::default_confirmation(),
            force_durable: Egress::default_force_durable(),
        }
    }

    /// Recreates this egress definition builder with the given name.
    pub fn with_name(self, name: impl AsRef<str>) -> Self {
        Self {
            name: Arc::from(name.as_ref()),
            ..self
        }
    }

    /// Recreates this egress definition builder with the given exchange name.
    pub fn with_exchange(self, exchange: impl AsRef<str>) -> Self {
        Self {
            exchange: Arc::from(exchange.as_ref()),
            ..self
        }
    }

    /// Recreates this egress definition builder with the given routing key.
    pub fn with_routing_key(self, routing_key: impl AsRef<str>) -> Self {
        Self {
            routing_key: Arc::from(routing_key.as_ref()),
            ..self
        }
    }

    /// Recreates this egress definition builder with the given confirmation
    /// level.
    pub fn with_confirmation(self, confirmation: ConfirmationLevel) -> Self {
        Self {
            confirmation,
            ..self
        }
    }

    /// Recreates this egress definition builder with the given `force_durable`
    /// flag.
    pub fn with_force_durable(self, force_durable: bool) -> Self {
        Self {
            force_durable,
            ..self
        }
    }

    /// Finalizes the builder, validates its state, and, assuming valid state,
    /// returns the [`Ingress`].
    ///
    /// In case the exchange name is the name of one of the known built-in RabbitMQ
    /// exchanges, some validation is applied on the given values. The built-in exchanges
    /// either require a non-empty routing key, or, on the opposite, ignore the routing
    /// key. For built-in exchanges, this method will return an error if the given routing
    /// key does not match the requirement.
    pub fn build(self) -> Result<Egress, EgressError> {
        self.validate()?;

        Ok(Egress {
            name: self.name,
            exchange: self.exchange,
            routing_key: self.routing_key,
            confirmation: self.confirmation,
            force_durable: self.force_durable,
        })
    }

    /// Validates whether the given combination of exchange name and routing key make
    /// sense.
    fn validate(&self) -> Result<(), EgressError> {
        if let Some(builtin_exchange) = Exchange::try_builtin_named(&self.exchange) {
            match builtin_exchange.kind() {
                ExchangeKind::Direct | ExchangeKind::Topic | ExchangeKind::HashKey => {
                    if self.routing_key.is_empty() {
                        return Err(EgressError::ExchangeRequiresRoutingKey {
                            egress: self.name.to_string(),
                            exchange: builtin_exchange,
                        });
                    };
                }
                ExchangeKind::Fanout | ExchangeKind::Headers | ExchangeKind::HashId => {
                    if !self.routing_key.is_empty() {
                        return Err(EgressError::ExchangeCannotHaveRoutingKey {
                            egress: self.name.to_string(),
                            exchange: builtin_exchange,
                            routing_key: self.routing_key.to_string(),
                        });
                    }
                }
            };
        }

        Ok(())
    }
}

impl Egress {
    fn default_name() -> &'static str {
        "default"
    }

    fn default_exchange() -> &'static str {
        ""
    }

    fn default_routing_key() -> &'static str {
        ""
    }

    fn default_confirmation() -> ConfirmationLevel {
        ConfirmationLevel::Transmitted
    }

    fn default_force_durable() -> bool {
        false
    }
}

#[cfg(test)]
impl Default for Egress {
    fn default() -> Self {
        Self {
            name: Arc::from(""),
            exchange: Arc::from(Self::default_exchange()),
            routing_key: Arc::from(Self::default_routing_key()),
            confirmation: Self::default_confirmation(),
            force_durable: Self::default_force_durable(),
        }
    }
}

/// Represents the various error states of a RabbitMQ egress definition.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum EgressError {
    /// Indicates the absence of a routing key where it is required.
    #[error(
        "invalid configuration for egress '{egress}' with built-in exchange '{exchange}' ({exchange:?}, type '{}'): expected routing key, found none/empty",
        .exchange.kind(),
    )]
    ExchangeRequiresRoutingKey {
        /// Egress name
        egress: String,
        /// Built-in exchange that requires a routing key
        exchange: Exchange,
    },

    /// Indicates the presence of a routing key where it is ignored.
    #[error(
        "invalid configuration for egress '{egress}' with built-in exchange '{exchange}' ({exchange:?}, type '{}'): expected no/empty routing key, found '{routing_key}'",
        .exchange.kind(),
    )]
    ExchangeCannotHaveRoutingKey {
        /// Egress name
        egress: String,
        /// Built-in exchange that ignores a routing key
        exchange: Exchange,
        /// Given routing key
        routing_key: String,
    },
}

impl AsRef<Egress> for Egress {
    fn as_ref(&self) -> &Egress {
        self
    }
}

impl AsRef<EgressLandscape> for EgressLandscape {
    fn as_ref(&self) -> &EgressLandscape {
        self
    }
}

const _: () = {
    impl<S> FromIterator<(S, Egress)> for EgressLandscape
    where
        S: Into<Slug>,
    {
        fn from_iter<T: IntoIterator<Item = (S, Egress)>>(iter: T) -> Self {
            let egresses = iter.into_iter().collect();

            Self { egresses }
        }
    }

    impl<const N: usize, S> From<[(S, Egress); N]> for EgressLandscape
    where
        S: Into<Slug>,
    {
        fn from(value: [(S, Egress); N]) -> Self {
            value.into_iter().collect()
        }
    }
};

const _: () = {
    impl<'de> Deserialize<'de> for EgressLandscape {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(EgressLandscapeVisitor)
        }
    }

    struct EgressLandscapeVisitor;

    impl<'de> Visitor<'de> for EgressLandscapeVisitor {
        type Value = EgressLandscape;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ egress landscape")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let grouped = Slug::group_map(map)?;
            let mut egresses = HashMap::with_capacity(grouped.len());

            for (key, value) in grouped {
                let seed = EgressSeed {
                    name: key.original(),
                };
                let handle = seed.deserialize(value).map_err(Error::custom)?;
                egresses.insert(key, handle);
            }

            Ok(EgressLandscape {
                egresses: SlugMap::new(egresses),
            })
        }
    }

    struct EgressSeed<'a> {
        name: &'a str,
    }

    impl<'de> DeserializeSeed<'de> for EgressSeed<'_> {
        type Value = Egress;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(EgressSeedVisitor { name: self.name })
        }
    }

    struct EgressSeedVisitor<'a> {
        name: &'a str,
    }

    impl<'de> Visitor<'de> for EgressSeedVisitor<'_> {
        type Value = Egress;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ egress or a string routing key")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Egress::builder()
                .with_name(self.name)
                .with_routing_key(value)
                .build()
                .map_err(Error::custom)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            visit_egress(map, Some(self.name))
        }
    }

    impl<'de> Deserialize<'de> for Egress {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(EgressVisitor)
        }
    }

    struct EgressVisitor;

    impl<'de> Visitor<'de> for EgressVisitor {
        type Value = Egress;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ egress")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            visit_egress(map, None)
        }
    }

    fn visit_egress<'de, A>(mut map: A, known_name: Option<&str>) -> Result<Egress, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut name: Option<String> = None;
        let mut exchange: Option<String> = None;
        let mut routing_key: Option<String> = None;
        let mut confirmation = None;
        let mut force_durable = None;

        while let Some(key) = map.next_key()? {
            match key {
                EgressField::name => key.poll(&mut map, &mut name)?,
                EgressField::exchange => key.poll(&mut map, &mut exchange)?,
                EgressField::routing_key => key.poll(&mut map, &mut routing_key)?,
                EgressField::confirmation => key.poll(&mut map, &mut confirmation)?,
                EgressField::force_durable => key.poll(&mut map, &mut force_durable)?,
                EgressField::__ignore => map.next_value()?,
            };
        }

        let name = match known_name {
            Some(known_name) => known_name,
            None => name.as_deref().unwrap_or_else(|| Egress::default_name()),
        };

        let mut builder = Egress::builder();

        let exchange = exchange
            .as_deref()
            .unwrap_or_else(|| Egress::default_exchange());
        let routing_key = routing_key
            .as_deref()
            .unwrap_or_else(|| Egress::default_routing_key());

        builder = builder
            .with_name(name)
            .with_exchange(exchange)
            .with_routing_key(routing_key);

        if let Some(confirmation) = confirmation {
            builder = builder.with_confirmation(confirmation);
        }

        if let Some(force_durable) = force_durable {
            builder = builder.with_force_durable(force_durable);
        }

        Ok(builder.build().map_err(Error::custom)?)
    }

    impl_deserialize_field!(
        EgressField,
        strut_deserialize::Slug::eq_as_slugs,
        name,
        exchange,
        routing_key,
        confirmation | confirmation_level,
        force_durable,
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
        let actual_output = serde_yml::from_str::<Egress>(input);

        // Then
        assert!(actual_output.is_err());
    }

    #[test]
    fn deserialize_from_string() {
        // Given
        let input = "\"test_egress\"";

        // When
        let actual_output = serde_yml::from_str::<Egress>(input);

        // Then
        assert!(actual_output.is_err());
    }

    #[test]
    fn deserialize_from_name() {
        // Given
        let input = r#"
name: test_egress
"#;

        // When
        let actual_output = serde_yml::from_str::<Egress>(input);

        // Then
        assert!(actual_output.is_err());
    }

    #[test]
    fn deserialize_from_name_and_exchange() {
        // Given
        let input = r#"
name: test_egress
exchange: amq.fanout
"#;
        let expected_output = Egress {
            name: "test_egress".into(),
            exchange: Exchange::AmqFanout.name().into(),
            ..Default::default()
        };

        // When
        let actual_output = serde_yml::from_str::<Egress>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_from_name_and_routing_key() {
        // Given
        let input = r#"
name: test_egress
routing_key: test_routing_key
"#;
        let expected_output = Egress {
            name: "test_egress".into(),
            routing_key: "test_routing_key".into(),
            ..Default::default()
        };

        // When
        let actual_output = serde_yml::from_str::<Egress>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_from_full() {
        // Given
        let input = r#"
extra_field: ignored
name: test_egress
exchange: amq.topic
routing_key: test_routing_key
confirmation: routed
force_durable: true
"#;
        let expected_output = Egress {
            name: "test_egress".into(),
            exchange: Exchange::AmqTopic.name().into(),
            routing_key: "test_routing_key".into(),
            confirmation: ConfirmationLevel::Routed,
            force_durable: true,
            ..Default::default()
        };

        // When
        let actual_output = serde_yml::from_str::<Egress>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn exchange_requires_routing_key() {
        // Given
        let expected_output = EgressError::ExchangeRequiresRoutingKey {
            egress: "test_egress".into(),
            exchange: Exchange::AmqTopic,
        };

        // When
        let actual_output = Egress::builder()
            .with_name("test_egress")
            .with_exchange(Exchange::AmqTopic.name())
            .with_routing_key("")
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_exchange_requires_routing_key() {
        // Given
        let input = r#"
name: test_egress
exchange: amq.topic
"#;
        let expected_output = EgressError::ExchangeRequiresRoutingKey {
            egress: "test_egress".into(),
            exchange: Exchange::AmqTopic,
        };

        // When
        let actual_output = serde_yml::from_str::<Egress>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn exchange_cannot_have_routing_key() {
        // Given
        let expected_output = EgressError::ExchangeCannotHaveRoutingKey {
            egress: "test_egress".into(),
            exchange: Exchange::AmqHeaders,
            routing_key: "test_routing_key".into(),
        };

        // When
        let actual_output = Egress::builder()
            .with_name("test_egress")
            .with_exchange(Exchange::AmqHeaders.name())
            .with_routing_key("test_routing_key")
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_exchange_cannot_have_routing_key() {
        // Given
        let input = r#"
name: test_egress
exchange: amq.headers
routing_key: test_routing_key
"#;
        let expected_output = EgressError::ExchangeCannotHaveRoutingKey {
            egress: "test_egress".into(),
            exchange: Exchange::AmqHeaders,
            routing_key: "test_routing_key".into(),
        };

        // When
        let actual_output = serde_yml::from_str::<Egress>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn deserialize_landscape_from_empty() {
        // Given
        let input = "";
        let expected_output = EgressLandscape::default();

        // When
        let actual_output = serde_yml::from_str::<EgressLandscape>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_landscape_from_full() {
        // Given
        let input = r#"
test_egress_a: test_routing_key_a
test_egress_b:
    exchange: test_exchange_b
    routing_key: test_routing_key_b
"#;
        let expected_output = EgressLandscape::from([
            (
                "test_egress_a",
                Egress::builder()
                    .with_name("test_egress_a")
                    .with_exchange("")
                    .with_routing_key("test_routing_key_a")
                    .build()
                    .unwrap(),
            ),
            (
                "test_egress_b",
                Egress::builder()
                    .with_name("test_egress_b")
                    .with_exchange("test_exchange_b")
                    .with_routing_key("test_routing_key_b")
                    .build()
                    .unwrap(),
            ),
        ]);

        // When
        let actual_output = serde_yml::from_str::<EgressLandscape>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }
}
