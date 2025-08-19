use crate::{
    AckingBehavior, Exchange, ExchangeKind, FinalizationKind, Header, HeadersMatchingBehavior,
    Queue,
};
use humantime::parse_duration;
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;
use std::num::{NonZeroU16, NonZeroUsize};
use std::sync::Arc;
use std::time::Duration;
use strut_deserialize::{OneOrMany, Slug, SlugMap};
use strut_factory::impl_deserialize_field;
use thiserror::Error;

pub mod queue;

/// Represents a collection of uniquely named [`Ingress`] definitions.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct IngressLandscape {
    ingresses: SlugMap<Ingress>,
}

/// Defines an inbound path for messages being consumed from a RabbitMQ cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ingress {
    name: Arc<str>,
    exchange: Exchange,
    queue: Queue,
    durable: bool,
    exclusive: bool,
    auto_delete: bool,
    batch_size: NonZeroUsize,
    batch_timeout: Duration,
    prefetch_count: Option<NonZeroU16>,
    acking_behavior: AckingBehavior,
    gibberish_behavior: FinalizationKind,
    // Exchange kind-specific configuration:
    binding_keys: HashSet<String>,             // direct, topic
    binding_headers: HashMap<String, Header>,  // headers
    headers_behavior: HeadersMatchingBehavior, // headers
}

impl IngressLandscape {
    /// Reports whether this landscape contains a [`Ingress`] with the
    /// given unique name.
    pub fn contains(&self, name: impl AsRef<str>) -> bool {
        self.ingresses.contains_key(name.as_ref())
    }

    /// Retrieves `Some` reference to a [`Ingress`] from this landscape
    /// under the given name, or `None`, if the name is not present in the
    /// landscape.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&Ingress> {
        self.ingresses.get(name.as_ref())
    }

    /// Retrieves a reference to a [`Ingress`] from this landscape under
    /// the given name. Panics if the name is not present in the collection.
    pub fn expect(&self, name: impl AsRef<str>) -> &Ingress {
        let name = name.as_ref();

        self.get(name)
            .unwrap_or_else(|| panic!("requested an undefined RabbitMQ ingress '{}'", name))
    }
}

impl Ingress {
    /// Creates a new [`IngressBuilder`].
    pub fn builder() -> IngressBuilder {
        IngressBuilder::new()
    }
}

impl Ingress {
    /// Reports the ingress name for this definition.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the exchange definition as part of this ingress definition.
    pub fn exchange(&self) -> &Exchange {
        &self.exchange
    }

    /// Returns the queue definition as part of this ingress definition.
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Reports the ingress `durable` flag for this definition.
    pub fn durable(&self) -> bool {
        self.durable
    }

    /// Reports the ingress `exclusive` flag for this definition.
    pub fn exclusive(&self) -> bool {
        self.exclusive
    }

    /// Reports the ingress `auto_delete` flag for this definition.
    pub fn auto_delete(&self) -> bool {
        self.auto_delete
    }

    /// Reports the desired `no_ack` value for a consumer based on this ingress
    /// definition.
    ///
    /// This boolean value is recognized by RabbitMQ and can be a bit misleading:
    ///
    /// - `false` = messages must be acknowledged manually by the client.
    /// - `true` = messages are acknowledged automatically by the broker,
    pub fn no_ack(&self) -> bool {
        match self.acking_behavior {
            AckingBehavior::Manual => false,
            AckingBehavior::Auto => true,
        }
    }

    /// Reports whether the messages delivered by a consumer based on this
    /// ingress definition are delivered in pending state (need to be manually
    /// finalized) or are delivered pre-finalized (pre-acknowledged on delivery).
    pub fn delivers_pending(&self) -> bool {
        match self.acking_behavior {
            AckingBehavior::Manual => true,
            AckingBehavior::Auto => false,
        }
    }

    /// Reports the ingress batch size for this definition.
    ///
    /// The [`Subscriber`] supports batch-consuming messages. After the first
    /// message of the batch is consumed, any message that arrive within the
    /// [timeout](Ingress::batch_timeout) will be appended to the same batch,
    /// unless this size limit is reached.
    pub fn batch_size(&self) -> NonZeroUsize {
        self.batch_size
    }

    /// Reports the ingress batch timeout for this definition.
    ///
    /// The [`Subscriber`] supports batch-consuming messages. After the first
    /// message of the batch is consumed, any message that arrive within this
    /// timeout will be appended to the same batch, unless the
    /// [size limit](Ingress::batch_size) is reached.
    pub fn batch_timeout(&self) -> Duration {
        self.batch_timeout
    }

    /// Reports the ingress prefetch count for this definition.
    pub fn prefetch_count(&self) -> Option<NonZeroU16> {
        self.prefetch_count
    }

    /// Reports the ingress acking behavior for this definition.
    pub fn acking_behavior(&self) -> AckingBehavior {
        self.acking_behavior
    }

    /// Reports the ingress gibberish behavior for this definition.
    pub fn gibberish_behavior(&self) -> FinalizationKind {
        self.gibberish_behavior
    }

    /// Reports the ingress binding keys for this definition.
    pub fn binding_keys(&self) -> &HashSet<String> {
        &self.binding_keys
    }

    /// Reports the ingress binding headers for this definition.
    pub fn binding_headers(&self) -> &HashMap<String, Header> {
        &self.binding_headers
    }

    /// Reports the ingress headers behavior for this definition.
    pub fn headers_behavior(&self) -> HeadersMatchingBehavior {
        self.headers_behavior
    }
}

/// Temporary struct for accumulating ingress configuration before finalizing it
/// into a [`Ingress`]. This builder intends to apply validation only at
/// meaningful states of the configuration, as opposed to every intermediary
/// state.
#[derive(Debug)]
pub struct IngressBuilder {
    name: Arc<str>,
    exchange: Exchange,
    queue: Queue,
    durable: bool,
    exclusive: bool,
    auto_delete: bool,
    batch_size: NonZeroUsize,
    batch_timeout: Duration,
    prefetch_count: Option<NonZeroU16>,
    acking_behavior: AckingBehavior,
    gibberish_behavior: FinalizationKind,
    // Exchange kind-specific configuration:
    binding_keys: HashSet<String>,             // direct, topic
    binding_headers: HashMap<String, Header>,  // headers
    headers_behavior: HeadersMatchingBehavior, // headers
}

impl IngressBuilder {
    /// Creates a new [`Ingress`] builder.
    pub fn new() -> Self {
        Self {
            name: Arc::from(Ingress::default_name()),
            exchange: Exchange::default(),
            queue: Queue::default(),
            durable: Ingress::default_durable(),
            exclusive: Ingress::default_exclusive(),
            auto_delete: Ingress::default_auto_delete(),
            batch_size: Ingress::default_batch_size(),
            batch_timeout: Ingress::default_batch_timeout(),
            prefetch_count: Ingress::default_prefetch_count(),
            acking_behavior: Ingress::default_acking_behavior(),
            gibberish_behavior: Ingress::default_gibberish_behavior(),
            binding_keys: Ingress::default_binding_keys(),
            binding_headers: Ingress::default_binding_headers(),
            headers_behavior: Ingress::default_headers_behavior(),
        }
    }

    /// Recreates this ingress definition builder with the given name.
    pub fn with_name(self, name: impl AsRef<str>) -> Self {
        Self {
            name: Arc::from(name.as_ref()),
            ..self
        }
    }

    /// Recreates this ingress definition builder with the given exchange.
    pub fn with_exchange(self, exchange: Exchange) -> Self {
        Self { exchange, ..self }
    }

    /// Recreates this ingress definition builder with the given queue.
    pub fn with_queue(self, queue: Queue) -> Self {
        Self { queue, ..self }
    }

    /// Recreates this ingress definition builder with a queue with the given
    /// name.
    pub fn with_queue_named(self, queue: impl AsRef<str>) -> Self {
        Self {
            queue: Queue::named(queue),
            ..self
        }
    }

    /// Recreates this ingress definition builder with the given `durable` flag.
    pub fn with_durable(self, durable: bool) -> Self {
        Self { durable, ..self }
    }

    /// Recreates this ingress definition builder with the given `exclusive` flag.
    pub fn with_exclusive(self, exclusive: bool) -> Self {
        Self { exclusive, ..self }
    }

    /// Recreates this ingress definition builder with the given `auto_delete` flag.
    pub fn with_auto_delete(self, auto_delete: bool) -> Self {
        Self {
            auto_delete,
            ..self
        }
    }

    /// Recreates this ingress definition builder with the given batch size.
    pub fn with_batch_size(self, batch_size: NonZeroUsize) -> Self {
        Self { batch_size, ..self }
    }

    /// Recreates this ingress definition builder with the given batch timeout.
    pub fn with_batch_timeout(self, batch_timeout: Duration) -> Self {
        Self {
            batch_timeout,
            ..self
        }
    }

    /// Recreates this ingress definition builder with the given prefetch count.
    pub fn with_prefetch_count(self, prefetch_count: Option<NonZeroU16>) -> Self {
        Self {
            prefetch_count,
            ..self
        }
    }

    /// Recreates this ingress definition builder with the given acking behavior.
    pub fn with_acking_behavior(self, acking_behavior: AckingBehavior) -> Self {
        Self {
            acking_behavior,
            ..self
        }
    }

    /// Recreates this ingress definition builder with the given gibberish behavior.
    pub fn with_gibberish_behavior(self, gibberish_behavior: FinalizationKind) -> Self {
        Self {
            gibberish_behavior,
            ..self
        }
    }

    /// Recreates this ingress definition builder, adding the given binding key
    /// to the ones already included.
    pub fn with_binding_key(self, binding_key: impl Into<String>) -> Self {
        let mut binding_keys = self.binding_keys;
        binding_keys.insert(binding_key.into());

        Self {
            binding_keys,
            ..self
        }
    }

    /// Recreates this ingress definition builder with the given binding keys.
    ///
    /// This will replace any previously
    /// [added](IngressBuilder::with_binding_key) binding keys.
    pub fn with_replaced_binding_keys(self, binding_keys: HashSet<String>) -> Self {
        Self {
            binding_keys,
            ..self
        }
    }

    /// Recreates this ingress definition builder, adding the given binding
    /// header to the ones already included.
    pub fn with_binding_header<V>(self, key: impl Into<String>, value: V) -> Self
    where
        Header: From<V>,
    {
        let mut binding_headers = self.binding_headers;
        binding_headers.insert(key.into(), Header::from(value));

        Self {
            binding_headers,
            ..self
        }
    }

    /// Recreates this ingress definition builder with the given binding headers.
    ///
    /// This will replace any previously
    /// [added](IngressBuilder::with_binding_header) binding headers.
    pub fn with_replaced_binding_headers(self, binding_headers: HashMap<String, Header>) -> Self {
        Self {
            binding_headers,
            ..self
        }
    }

    /// Recreates this ingress definition builder with the headers matching
    /// behavior set to `all` (all headers must match).
    pub fn with_matching_all_headers(self) -> Self {
        Self {
            headers_behavior: HeadersMatchingBehavior::All,
            ..self
        }
    }

    /// Recreates this ingress definition builder with the headers matching
    /// behavior set to `any` (at least one header must match).
    pub fn with_matching_any_headers(self) -> Self {
        Self {
            headers_behavior: HeadersMatchingBehavior::Any,
            ..self
        }
    }

    /// Recreates this ingress definition builder with the given headers behavior.
    pub fn with_headers_behavior(self, headers_behavior: HeadersMatchingBehavior) -> Self {
        Self {
            headers_behavior,
            ..self
        }
    }

    /// Finalizes the builder, validates its state, and, assuming valid state,
    /// returns the [`Ingress`].
    pub fn build(mut self) -> Result<Ingress, IngressError> {
        // At the last moment, slide in a possible implicit binding key
        if let Some(implicit_binding_key) = self.maybe_implicit_binding_key() {
            self.binding_keys = implicit_binding_key;
        }

        self.validate()?;

        Ok(Ingress {
            name: self.name,
            exchange: self.exchange,
            queue: self.queue,
            durable: self.durable,
            exclusive: self.exclusive,
            auto_delete: self.auto_delete,
            batch_size: self.batch_size,
            batch_timeout: self.batch_timeout,
            prefetch_count: self.prefetch_count,
            acking_behavior: self.acking_behavior,
            gibberish_behavior: self.gibberish_behavior,
            binding_keys: self.binding_keys,
            binding_headers: self.binding_headers,
            headers_behavior: self.headers_behavior,
        })
    }

    fn maybe_implicit_binding_key(&self) -> Option<HashSet<String>> {
        // Does the exchange allow explicit binding key?
        let exchange_allows_implicit_binding_key = match self.exchange.kind() {
            ExchangeKind::Direct | ExchangeKind::Topic => !self.exchange.is_default(),
            _ => false,
        };

        // Can we maybe use the queue name as a binding key?
        if exchange_allows_implicit_binding_key
            && self.binding_keys.is_empty()
            && !self.queue.is_empty()
        {
            // Use queue name also as a binding key
            return Some(HashSet::from([self.queue.name().to_string()]));
        }

        None
    }

    fn validate(&self) -> Result<(), IngressError> {
        if self.acking_behavior == AckingBehavior::Manual {
            if let Some(prefetch_count) = self.prefetch_count {
                if usize::from(self.batch_size) > (u16::from(prefetch_count) as usize) {
                    return Err(IngressError::BatchSizeGreaterThanPrefetchCount {
                        ingress: self.name.to_string(),
                        prefetch_count,
                        batch_size: self.batch_size,
                    });
                }
            } else {
                if self.batch_size > NonZeroUsize::MIN {
                    return Err(IngressError::BatchSizeWithoutPrefetchCount {
                        ingress: self.name.to_string(),
                        batch_size: self.batch_size,
                    });
                }
            }
        }

        if self.exchange.is_default() {
            self.validate_default_exchange()?;
        } else {
            self.validate_non_default_exchange()?;
        }

        Ok(())
    }

    fn validate_default_exchange(&self) -> Result<(), IngressError> {
        // Ensure queue name is not empty
        if self.queue.is_empty() {
            return Err(IngressError::DefaultExchangeRequiresQueueName {
                ingress: self.name.to_string(),
            });
        }

        // Ensure there are no binding keys
        if !self.binding_keys.is_empty() {
            return Err(IngressError::DefaultExchangeCannotHaveBindingKeys {
                ingress: self.name.to_string(),
            });
        }

        // Ensure there are no binding headers
        if !self.binding_headers.is_empty() {
            return Err(IngressError::DefaultExchangeCannotHaveBindingHeaders {
                ingress: self.name.to_string(),
            });
        }

        Ok(())
    }

    fn validate_non_default_exchange(&self) -> Result<(), IngressError> {
        match self.exchange.kind() {
            ExchangeKind::Direct | ExchangeKind::Topic => self.validate_binding_keys()?,
            ExchangeKind::Headers => self.validate_binding_headers()?,
            ExchangeKind::Fanout | ExchangeKind::HashKey | ExchangeKind::HashId => {
                self.validate_no_bindings()?
            }
        };

        Ok(())
    }

    fn validate_binding_keys(&self) -> Result<(), IngressError> {
        self.validate_no_binding_headers()?;

        if self.binding_keys.is_empty() {
            return Err(IngressError::ExchangeKindRequiresBindingKeys {
                ingress: self.name.to_string(),
                kind: self.exchange.kind(),
            });
        }

        for key in &self.binding_keys {
            if key.is_empty() {
                return Err(IngressError::ExchangeKindCannotHaveEmptyBindingKey {
                    ingress: self.name.to_string(),
                    kind: self.exchange.kind(),
                });
            }
        }

        Ok(())
    }

    fn validate_binding_headers(&self) -> Result<(), IngressError> {
        self.validate_no_binding_keys()?;

        if self.binding_headers.is_empty() {
            return Err(IngressError::ExchangeKindRequiresBindingHeaders {
                ingress: self.name.to_string(),
                kind: self.exchange.kind(),
            });
        }

        for (key, value) in &self.binding_headers {
            if key.is_empty() || value.is_empty() {
                return Err(IngressError::ExchangeKindCannotHaveEmptyBindingHeader {
                    ingress: self.name.to_string(),
                    kind: self.exchange.kind(),
                });
            }
        }

        Ok(())
    }

    fn validate_no_binding_keys(&self) -> Result<(), IngressError> {
        if !self.binding_keys.is_empty() {
            return Err(IngressError::ExchangeKindCannotHaveBindingKeys {
                ingress: self.name.to_string(),
                kind: self.exchange.kind(),
            });
        }

        Ok(())
    }

    fn validate_no_binding_headers(&self) -> Result<(), IngressError> {
        if !self.binding_headers.is_empty() {
            return Err(IngressError::ExchangeKindCannotHaveBindingHeaders {
                ingress: self.name.to_string(),
                kind: self.exchange.kind(),
            });
        }

        Ok(())
    }

    fn validate_no_bindings(&self) -> Result<(), IngressError> {
        self.validate_no_binding_keys()?;
        self.validate_no_binding_headers()?;

        Ok(())
    }
}

impl Ingress {
    fn default_name() -> &'static str {
        "default"
    }

    fn default_durable() -> bool {
        false
    }

    fn default_exclusive() -> bool {
        false
    }

    fn default_auto_delete() -> bool {
        false
    }

    fn default_batch_size() -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn default_batch_timeout() -> Duration {
        Duration::from_millis(250)
    }

    fn default_prefetch_count() -> Option<NonZeroU16> {
        None
    }

    fn default_acking_behavior() -> AckingBehavior {
        AckingBehavior::Manual
    }

    fn default_gibberish_behavior() -> FinalizationKind {
        FinalizationKind::Complete
    }

    fn default_binding_keys() -> HashSet<String> {
        HashSet::default()
    }

    fn default_binding_headers() -> HashMap<String, Header> {
        HashMap::default()
    }

    fn default_headers_behavior() -> HeadersMatchingBehavior {
        HeadersMatchingBehavior::All
    }
}

#[cfg(test)]
impl Default for Ingress {
    fn default() -> Self {
        Self {
            name: Arc::from(""),
            exchange: Exchange::default(),
            queue: Queue::default(),
            durable: Self::default_durable(),
            exclusive: Self::default_exclusive(),
            auto_delete: Self::default_auto_delete(),
            batch_size: Self::default_batch_size(),
            batch_timeout: Self::default_batch_timeout(),
            prefetch_count: Self::default_prefetch_count(),
            acking_behavior: Self::default_acking_behavior(),
            gibberish_behavior: Self::default_gibberish_behavior(),
            binding_keys: Self::default_binding_keys(),
            binding_headers: Self::default_binding_headers(),
            headers_behavior: Self::default_headers_behavior(),
        }
    }
}

/// Represents the various error states of a RabbitMQ ingress definition.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum IngressError {
    /// Indicates batch size greater than prefetch count
    #[error(
        "invalid batch size configuration for ingress '{ingress}' with prefetch count of {prefetch_count}: expected <= {prefetch_count}, found {batch_size}"
    )]
    BatchSizeGreaterThanPrefetchCount {
        /// Ingress name
        ingress: String,
        /// Batch size
        batch_size: NonZeroUsize,
        /// Prefetch count,
        prefetch_count: NonZeroU16,
    },

    /// Indicates batch size on an ingress without prefetch count
    #[error(
        "invalid batch size configuration for ingress '{ingress}' without prefetch count: expected 1, found {batch_size}"
    )]
    BatchSizeWithoutPrefetchCount {
        /// Ingress name
        ingress: String,
        /// Batch size
        batch_size: NonZeroUsize,
    },

    /// Indicates the absence of a queue name where it is required.
    #[error(
        "invalid configuration for ingress '{ingress}' with default exchange: expected queue name, found none/empty"
    )]
    DefaultExchangeRequiresQueueName {
        /// Ingress name
        ingress: String,
    },

    /// Indicates the presence of binding keys on a default exchange, which doesn’t allow them.
    #[error(
        "invalid configuration for ingress '{ingress}' with default exchange: expected no binding keys, found at least one"
    )]
    DefaultExchangeCannotHaveBindingKeys {
        /// Ingress name
        ingress: String,
    },

    /// Indicates the presence of binding headers on a default exchange, which doesn’t allow them.
    #[error(
        "invalid configuration for ingress '{ingress}' with default exchange: expected no binding headers, found at least one"
    )]
    DefaultExchangeCannotHaveBindingHeaders {
        /// Ingress name
        ingress: String,
    },

    /// Indicates the absence of binding keys on an exchange kind that requires them.
    #[error(
        "invalid configuration for ingress '{ingress}' with exchange of type '{kind:?}': expected at least one binding key, found none"
    )]
    ExchangeKindRequiresBindingKeys {
        /// Ingress name
        ingress: String,
        /// Exchange kind
        kind: ExchangeKind,
    },

    /// Indicates the presence of binding keys on an exchange kind that ignores them.
    #[error(
        "invalid configuration for ingress '{ingress}' with exchange of type '{kind:?}': expected no binding keys, found at least one"
    )]
    ExchangeKindCannotHaveBindingKeys {
        /// Ingress name
        ingress: String,
        /// Exchange kind
        kind: ExchangeKind,
    },

    /// Indicates the absence of binding headers on an exchange kind that requires them.
    #[error(
        "invalid configuration for ingress '{ingress}' with exchange of type '{kind:?}': expected at least one binding header, found none"
    )]
    ExchangeKindRequiresBindingHeaders {
        /// Ingress name
        ingress: String,
        /// Exchange kind
        kind: ExchangeKind,
    },

    /// Indicates the presence of binding headers on an exchange kind that ignores them.
    #[error(
        "invalid configuration for ingress '{ingress}' with exchange of type '{kind:?}': expected no binding headers, found at least one"
    )]
    ExchangeKindCannotHaveBindingHeaders {
        /// Ingress name
        ingress: String,
        /// Exchange kind
        kind: ExchangeKind,
    },

    /// Indicates the presence of an empty binding key on an exchange kind that requires them.
    #[error(
        "invalid configuration for ingress '{ingress}' with exchange of type '{kind:?}': expected non-empty binding keys, found an empty one"
    )]
    ExchangeKindCannotHaveEmptyBindingKey {
        /// Ingress name
        ingress: String,
        /// Exchange kind
        kind: ExchangeKind,
    },

    /// Indicates the presence of an empty binding header on an exchange kind that requires them.
    #[error(
        "invalid configuration for ingress '{ingress}' with exchange of type '{kind:?}': expected non-empty binding header keys and values, found an empty one"
    )]
    ExchangeKindCannotHaveEmptyBindingHeader {
        /// Ingress name
        ingress: String,
        /// Exchange kind
        kind: ExchangeKind,
    },
}

impl AsRef<Ingress> for Ingress {
    fn as_ref(&self) -> &Ingress {
        self
    }
}

impl AsRef<IngressLandscape> for IngressLandscape {
    fn as_ref(&self) -> &IngressLandscape {
        self
    }
}

const _: () = {
    impl<S> FromIterator<(S, Ingress)> for IngressLandscape
    where
        S: Into<Slug>,
    {
        fn from_iter<T: IntoIterator<Item = (S, Ingress)>>(iter: T) -> Self {
            let ingresses = iter.into_iter().collect();

            Self { ingresses }
        }
    }

    impl<const N: usize, S> From<[(S, Ingress); N]> for IngressLandscape
    where
        S: Into<Slug>,
    {
        fn from(value: [(S, Ingress); N]) -> Self {
            value.into_iter().collect()
        }
    }
};

const _: () = {
    impl<'de> Deserialize<'de> for IngressLandscape {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(IngressLandscapeVisitor)
        }
    }

    struct IngressLandscapeVisitor;

    impl<'de> Visitor<'de> for IngressLandscapeVisitor {
        type Value = IngressLandscape;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ ingress landscape")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let grouped = Slug::group_map(map)?;
            let mut ingresses = HashMap::with_capacity(grouped.len());

            for (key, value) in grouped {
                let seed = IngressSeed {
                    name: key.original(),
                };
                let handle = seed.deserialize(value).map_err(Error::custom)?;
                ingresses.insert(key, handle);
            }

            Ok(IngressLandscape {
                ingresses: SlugMap::new(ingresses),
            })
        }
    }

    struct IngressSeed<'a> {
        name: &'a str,
    }

    impl<'de> DeserializeSeed<'de> for IngressSeed<'_> {
        type Value = Ingress;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(IngressSeedVisitor { name: self.name })
        }
    }

    struct IngressSeedVisitor<'a> {
        name: &'a str,
    }

    impl<'de> Visitor<'de> for IngressSeedVisitor<'_> {
        type Value = Ingress;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ ingress or a string queue name")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ingress::builder()
                .with_name(self.name)
                .with_queue_named(value)
                .build()
                .map_err(Error::custom)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            visit_ingress(map, Some(self.name))
        }
    }

    impl<'de> Deserialize<'de> for Ingress {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(IngressVisitor)
        }
    }

    struct IngressVisitor;

    impl<'de> Visitor<'de> for IngressVisitor {
        type Value = Ingress;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ ingress")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            visit_ingress(map, None)
        }
    }

    fn visit_ingress<'de, A>(mut map: A, known_name: Option<&str>) -> Result<Ingress, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut name: Option<String> = None;
        let mut exchange = None;
        let mut queue = None;
        let mut durable = None;
        let mut exclusive = None;
        let mut auto_delete = None;
        let mut batch_size = None;
        let mut batch_timeout = None;
        let mut prefetch_count = None;
        let mut acking_behavior = None;
        let mut gibberish_behavior = None;
        let mut binding_keys: Option<OneOrMany<String>> = None;
        let mut binding_headers = None;
        let mut headers_behavior = None;

        while let Some(key) = map.next_key()? {
            match key {
                IngressField::name => key.poll(&mut map, &mut name)?,
                IngressField::exchange => key.poll(&mut map, &mut exchange)?,
                IngressField::queue => key.poll(&mut map, &mut queue)?,
                IngressField::durable => key.poll(&mut map, &mut durable)?,
                IngressField::exclusive => key.poll(&mut map, &mut exclusive)?,
                IngressField::auto_delete => key.poll(&mut map, &mut auto_delete)?,
                IngressField::batch_size => key.poll(&mut map, &mut batch_size)?,
                IngressField::batch_timeout => {
                    let duration_string = map.next_value::<String>()?;
                    let duration = parse_duration(&duration_string).map_err(Error::custom)?;
                    batch_timeout = Some(duration);
                    IgnoredAny
                }
                IngressField::prefetch_count => key.poll(&mut map, &mut prefetch_count)?,
                IngressField::acking_behavior => key.poll(&mut map, &mut acking_behavior)?,
                IngressField::gibberish_behavior => key.poll(&mut map, &mut gibberish_behavior)?,
                IngressField::binding_keys => key.poll(&mut map, &mut binding_keys)?,
                IngressField::binding_headers => key.poll(&mut map, &mut binding_headers)?,
                IngressField::headers_behavior => key.poll(&mut map, &mut headers_behavior)?,
                IngressField::__ignore => map.next_value()?,
            };
        }

        let name = match known_name {
            Some(known_name) => known_name,
            None => name.as_deref().unwrap_or_else(|| Ingress::default_name()),
        };

        let mut builder = Ingress::builder()
            .with_name(name)
            .with_exchange(exchange.unwrap_or_default())
            .with_queue(queue.unwrap_or_default());

        if let Some(durable) = durable {
            builder = builder.with_durable(durable);
        }
        if let Some(exclusive) = exclusive {
            builder = builder.with_exclusive(exclusive);
        }
        if let Some(auto_delete) = auto_delete {
            builder = builder.with_auto_delete(auto_delete);
        }
        if let Some(batch_size) = batch_size {
            builder = builder.with_batch_size(batch_size);
        }
        if let Some(batch_timeout) = batch_timeout {
            builder = builder.with_batch_timeout(batch_timeout);
        }
        if let Some(prefetch_count) = prefetch_count {
            builder = builder.with_prefetch_count(prefetch_count);
        }
        if let Some(acking_behavior) = acking_behavior {
            builder = builder.with_acking_behavior(acking_behavior);
        }
        if let Some(gibberish_behavior) = gibberish_behavior {
            builder = builder.with_gibberish_behavior(gibberish_behavior);
        }
        if let Some(binding_keys) = binding_keys {
            builder = builder.with_replaced_binding_keys(binding_keys.into());
        }
        if let Some(binding_headers) = binding_headers {
            builder = builder.with_replaced_binding_headers(binding_headers);
        }
        if let Some(headers_behavior) = headers_behavior {
            builder = builder.with_headers_behavior(headers_behavior);
        }

        builder.build().map_err(Error::custom)
    }

    impl_deserialize_field!(
        IngressField,
        strut_deserialize::Slug::eq_as_slugs,
        name,
        exchange,
        queue,
        durable,
        exclusive,
        auto_delete,
        batch_size,
        batch_timeout,
        prefetch_count | prefetch,
        acking_behavior | acking,
        gibberish_behavior | gibberish,
        binding_keys | binding_key,
        binding_headers | binding_header,
        headers_behavior | header_behavior,
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
        let actual_output = serde_yml::from_str::<Ingress>(input);

        // Then
        assert!(actual_output.is_err());
    }

    #[test]
    fn deserialize_from_string() {
        // Given
        let input = "\"test_ingress\"";

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input);

        // Then
        assert!(actual_output.is_err());
    }

    #[test]
    fn deserialize_from_name() {
        // Given
        let input = r#"
name: test_ingress
"#;

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input);

        // Then
        assert!(actual_output.is_err());
    }

    #[test]
    fn deserialize_from_name_and_exchange() {
        // Given
        let input = r#"
name: test_ingress
exchange: amq.fanout
"#;
        let expected_output = Ingress {
            name: "test_ingress".into(),
            exchange: Exchange::AmqFanout,
            ..Default::default()
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_from_name_and_routing_key() {
        // Given
        let input = r#"
name: test_ingress
queue: test_queue
"#;
        let expected_output = Ingress {
            name: "test_ingress".into(),
            queue: Queue::named("test_queue"),
            ..Default::default()
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_from_full() {
        // Given
        let input = r#"
extra_field: ignored
name: test_ingress
exchange: amq.topic
queue: test_queue
durable: true
exclusive: true
auto_delete: true
batch_size: 21
batch_timeout: 2s 150ms
prefetch_count: 42
acking_behavior: manual
gibberish_behavior: backwash
binding_keys:
  - test_binding_key_1
  - test_binding_key_2
binding_headers: {}
headers_behavior: any
"#;
        let expected_output = Ingress {
            name: "test_ingress".into(),
            exchange: Exchange::AmqTopic,
            queue: Queue::named("test_queue"),
            durable: true,
            exclusive: true,
            auto_delete: true,
            batch_size: NonZeroUsize::new(21).unwrap(),
            batch_timeout: Duration::from_millis(2150),
            prefetch_count: Some(NonZeroU16::new(42).unwrap()),
            acking_behavior: AckingBehavior::Manual,
            gibberish_behavior: FinalizationKind::Backwash,
            binding_keys: HashSet::from([
                "test_binding_key_1".to_string(),
                "test_binding_key_2".to_string(),
            ]),
            binding_headers: HashMap::new(),
            headers_behavior: HeadersMatchingBehavior::Any,
            ..Default::default()
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn default_exchange_requires_queue_name() {
        // Given
        let expected_output = IngressError::DefaultExchangeRequiresQueueName {
            ingress: "test_ingress".into(),
        };

        // When
        let actual_output = Ingress::builder()
            .with_name("test_ingress")
            .with_exchange(Exchange::Default)
            .with_queue(Queue::empty())
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_default_exchange_requires_queue_name() {
        // Given
        let input = r#"
name: test_ingress
exchange: ''
queue: ''
"#;
        let expected_output = IngressError::DefaultExchangeRequiresQueueName {
            ingress: "test_ingress".into(),
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn default_exchange_cannot_have_binding_keys() {
        // Given
        let expected_output = IngressError::DefaultExchangeCannotHaveBindingKeys {
            ingress: "test_ingress".into(),
        };

        // When
        let actual_output = Ingress::builder()
            .with_name("test_ingress")
            .with_exchange(Exchange::Default)
            .with_queue_named("test_queue")
            .with_binding_key("test_binding_key")
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_default_exchange_cannot_have_binding_keys() {
        // Given
        let input = r#"
name: test_ingress
exchange: ''
queue: test_queue
binding_keys:
  - test_binding_key
"#;
        let expected_output = IngressError::DefaultExchangeCannotHaveBindingKeys {
            ingress: "test_ingress".into(),
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn default_exchange_cannot_have_binding_headers() {
        // Given
        let expected_output = IngressError::DefaultExchangeCannotHaveBindingHeaders {
            ingress: "test_ingress".into(),
        };

        // When
        let actual_output = Ingress::builder()
            .with_name("test_ingress")
            .with_exchange(Exchange::Default)
            .with_queue_named("test_queue")
            .with_binding_header("test_binding_header", 42)
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_default_exchange_cannot_have_binding_headers() {
        // Given
        let input = r#"
name: test_ingress
exchange: ''
queue: test_queue
binding_headers:
    test_binding_header: '42'
"#;
        let expected_output = IngressError::DefaultExchangeCannotHaveBindingHeaders {
            ingress: "test_ingress".into(),
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn exchange_kind_requires_binding_keys() {
        // Given
        let expected_output = IngressError::ExchangeKindRequiresBindingKeys {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Direct,
        };

        // When
        let actual_output = Ingress::builder()
            .with_name("test_ingress")
            .with_exchange(Exchange::AmqDirect)
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_exchange_kind_requires_binding_keys() {
        // Given
        let input = r#"
name: test_ingress
exchange: amq.direct
"#;
        let expected_output = IngressError::ExchangeKindRequiresBindingKeys {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Direct,
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn exchange_kind_cannot_have_binding_keys() {
        // Given
        let expected_output = IngressError::ExchangeKindCannotHaveBindingKeys {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Headers,
        };

        // When
        let actual_output = Ingress::builder()
            .with_name("test_ingress")
            .with_exchange(Exchange::AmqHeaders)
            .with_queue_named("test_queue")
            .with_binding_key("test_binding_key")
            .with_binding_header("test_binding_header", 42)
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_exchange_kind_cannot_have_binding_keys() {
        // Given
        let input = r#"
name: test_ingress
exchange: amq.headers
queue: test_queue
binding_key: test_binding_key
binding_headers:
    test_binding_header: 42
"#;
        let expected_output = IngressError::ExchangeKindCannotHaveBindingKeys {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Headers,
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn exchange_kind_requires_binding_headers() {
        // Given
        let expected_output = IngressError::ExchangeKindRequiresBindingHeaders {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Headers,
        };

        // When
        let actual_output = Ingress::builder()
            .with_name("test_ingress")
            .with_exchange(Exchange::AmqHeaders)
            .with_queue_named("test_queue")
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_exchange_kind_requires_binding_headers() {
        // Given
        let input = r#"
name: test_ingress
exchange: amq.headers
queue: test_queue
"#;
        let expected_output = IngressError::ExchangeKindRequiresBindingHeaders {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Headers,
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn exchange_kind_cannot_have_binding_headers() {
        // Given
        let expected_output = IngressError::ExchangeKindCannotHaveBindingHeaders {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Topic,
        };

        // When
        let actual_output = Ingress::builder()
            .with_name("test_ingress")
            .with_exchange(Exchange::AmqTopic)
            .with_queue_named("test_queue")
            .with_binding_key("test_binding_key")
            .with_binding_header("test_binding_header", 42)
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_exchange_kind_cannot_have_binding_headers() {
        // Given
        let input = r#"
name: test_ingress
exchange: amq.topic
queue: test_queue
binding_key: test_binding_key
binding_headers:
    test_binding_header: 42
"#;
        let expected_output = IngressError::ExchangeKindCannotHaveBindingHeaders {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Topic,
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn exchange_kind_cannot_have_empty_binding_key() {
        // Given
        let expected_output = IngressError::ExchangeKindCannotHaveEmptyBindingKey {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Topic,
        };

        // When
        let actual_output = Ingress::builder()
            .with_name("test_ingress")
            .with_exchange(Exchange::AmqTopic)
            .with_queue_named("test_queue")
            .with_binding_key("")
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_exchange_kind_cannot_have_empty_binding_key() {
        // Given
        let input = r#"
name: test_ingress
exchange: amq.topic
queue: test_queue
binding_key: ''
"#;
        let expected_output = IngressError::ExchangeKindCannotHaveEmptyBindingKey {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Topic,
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap_err();

        // Then
        assert!(
            actual_output
                .to_string()
                .starts_with(&expected_output.to_string()),
        );
    }

    #[test]
    fn exchange_kind_cannot_have_empty_binding_header() {
        // Given
        let expected_output = IngressError::ExchangeKindCannotHaveEmptyBindingHeader {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Headers,
        };

        // When
        let actual_output = Ingress::builder()
            .with_name("test_ingress")
            .with_exchange(Exchange::AmqHeaders)
            .with_queue_named("test_queue")
            .with_binding_header("test_binding_header", "")
            .build()
            .unwrap_err();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_exchange_kind_cannot_have_empty_binding_header() {
        // Given
        let input = r#"
name: test_ingress
exchange: amq.headers
queue: test_queue
binding_header:
    test_binding_header: ''
"#;
        let expected_output = IngressError::ExchangeKindCannotHaveEmptyBindingHeader {
            ingress: "test_ingress".into(),
            kind: ExchangeKind::Headers,
        };

        // When
        let actual_output = serde_yml::from_str::<Ingress>(input).unwrap_err();

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
        let expected_output = IngressLandscape::default();

        // When
        let actual_output = serde_yml::from_str::<IngressLandscape>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_landscape_from_full() {
        // Given
        let input = r#"
test_ingress_a: test_queue_a
test_ingress_b:
    exchange: test_exchange_b
    queue: test_queue_b
    binding_key: test_binding_key_b
"#;
        let expected_output = IngressLandscape::from([
            (
                "test_ingress_a",
                Ingress::builder()
                    .with_name("test_ingress_a")
                    .with_exchange(Exchange::Default)
                    .with_queue_named("test_queue_a")
                    .build()
                    .unwrap(),
            ),
            (
                "test_ingress_b",
                Ingress::builder()
                    .with_name("test_ingress_b")
                    .with_exchange(Exchange::named("test_exchange_b").unwrap())
                    .with_queue_named("test_queue_b")
                    .with_binding_key("test_binding_key_b")
                    .build()
                    .unwrap(),
            ),
        ]);

        // When
        let actual_output = serde_yml::from_str::<IngressLandscape>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }
}
