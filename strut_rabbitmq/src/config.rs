use crate::{EgressLandscape, Handle, HandleCollection, IngressLandscape};
use serde::de::{Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_value::Value;
use std::collections::BTreeMap;
use std::fmt::Formatter;
use strut_factory::impl_deserialize_field;

/// Represents the application-level configuration section that covers everything
/// related to RabbitMQ connectivity:
///
/// - server URL and credentials ([`Handle`]),
/// - inbound message routing ([`Ingress`](crate::Ingress)),
/// - outbound message routing ([`Egress`](crate::Egress)).
///
/// This config comes with a custom [`Deserialize`] implementation, to support more
/// human-oriented textual configuration.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct RabbitMqConfig {
    default_handle: Handle,
    extra_handles: HandleCollection,
    ingress: IngressLandscape,
    egress: EgressLandscape,
}

impl RabbitMqConfig {
    /// Returns the default [`Handle`] for this configuration.
    pub fn default_handle(&self) -> &Handle {
        &self.default_handle
    }

    /// Returns the extra [`Handle`]s for this configuration.
    pub fn extra_handles(&self) -> &HandleCollection {
        &self.extra_handles
    }

    /// Returns the [`IngressLandscape`] for this configuration.
    pub fn ingress(&self) -> &IngressLandscape {
        &self.ingress
    }

    /// Returns the [`EgressLandscape`] for this configuration.
    pub fn egress(&self) -> &EgressLandscape {
        &self.egress
    }
}

impl AsRef<RabbitMqConfig> for RabbitMqConfig {
    fn as_ref(&self) -> &RabbitMqConfig {
        self
    }
}

const _: () = {
    impl<'de> Deserialize<'de> for RabbitMqConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(RabbitMqConfigVisitor)
        }
    }

    struct RabbitMqConfigVisitor;

    impl<'de> Visitor<'de> for RabbitMqConfigVisitor {
        type Value = RabbitMqConfig;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of application RabbitMQ configuration")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut default_handle = None;
            let mut extra_handles = None;
            let mut ingress = None;
            let mut egress = None;

            let mut discarded = BTreeMap::new();

            while let Some(key) = map.next_key::<Value>()? {
                let field = RabbitMqConfigField::deserialize(key.clone()).map_err(Error::custom)?;

                match field {
                    RabbitMqConfigField::default_handle => {
                        field.poll(&mut map, &mut default_handle)?
                    }
                    RabbitMqConfigField::extra_handles => {
                        field.poll(&mut map, &mut extra_handles)?
                    }
                    RabbitMqConfigField::ingress => field.poll(&mut map, &mut ingress)?,
                    RabbitMqConfigField::egress => field.poll(&mut map, &mut egress)?,
                    RabbitMqConfigField::__ignore => {
                        discarded.insert(key, map.next_value()?);
                        IgnoredAny
                    }
                };
            }

            if default_handle.is_none() {
                default_handle =
                    Some(Handle::deserialize(Value::Map(discarded)).map_err(Error::custom)?);
            }

            Ok(RabbitMqConfig {
                default_handle: default_handle.unwrap_or_default(),
                extra_handles: extra_handles.unwrap_or_default(),
                ingress: ingress.unwrap_or_default(),
                egress: egress.unwrap_or_default(),
            })
        }
    }

    impl_deserialize_field!(
        RabbitMqConfigField,
        strut_deserialize::Slug::eq_as_slugs,
        default_handle | default,
        extra_handles | extra | extras,
        ingress | inbound | incoming | subscriber | subscribers,
        egress | outbound | outgoing | publisher | publishers,
    );
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DsnChunks, Egress, Exchange, Ingress};
    use pretty_assertions::assert_eq;

    #[test]
    fn empty() {
        // Given
        let input = "";
        let expected_output = RabbitMqConfig::default();

        // When
        let actual_output = serde_yml::from_str::<RabbitMqConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn full() {
        // Given
        let input = r#"
host: custom-domain.com
port: 6879
user: test_user
vhost: /custom
extra:
  other_handle:
    vhost: /other
inbound:
  in_route:
    exchange: amq.topic
    queue: inbound_queue
    binding_key: inbound_binding_key
outbound:
  out_route:
    exchange: amq.fanout
"#;
        let expected_output = RabbitMqConfig {
            default_handle: Handle::new(
                "default",
                DsnChunks {
                    host: "custom-domain.com",
                    port: 6879,
                    user: "test_user",
                    vhost: "/custom",
                    ..Default::default()
                },
            ),
            extra_handles: HandleCollection::from([(
                "other_handle",
                Handle::new(
                    "other_handle",
                    DsnChunks {
                        vhost: "/other",
                        ..Default::default()
                    },
                ),
            )]),
            ingress: IngressLandscape::from([(
                "in_route",
                Ingress::builder()
                    .with_name("in_route")
                    .with_exchange(Exchange::AmqTopic)
                    .with_queue_named("inbound_queue")
                    .with_binding_key("inbound_binding_key")
                    .build()
                    .unwrap(),
            )]),
            egress: EgressLandscape::from([(
                "out_route",
                Egress::builder()
                    .with_name("out_route")
                    .with_exchange("amq.fanout")
                    .with_routing_key("")
                    .build()
                    .unwrap(),
            )]),
        };

        // When
        let actual_output = serde_yml::from_str::<RabbitMqConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }
}
