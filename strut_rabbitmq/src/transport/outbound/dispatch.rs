use crate::util::{
    Attempt, Morph, Push, PushAppId, PushClusterId, PushContentEncoding, PushContentType,
    PushCorrelationId, PushExpiration, PushKind, PushMessageId, PushReplyTo, PushUserId,
    RetrievePushMap,
};
use crate::DeliveryMode;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::{AMQPValue, FieldTable, ShortString};
use std::borrow::Cow;

/// Represents an **outgoing** RabbitMQ message.
///
/// This dispatch owns only the encoded bytes of the outgoing payload, but not
/// the payload itself. Furthermore, this dispatch provides no facilities for
/// encoding the payload.
///
/// It is possible to directly [create](crate::Envelope::dispatch_builder) a
/// [`DispatchBuilder`] from an [`Envelope`].
#[derive(Debug)]
pub struct Dispatch {
    bytes: Vec<u8>,
    properties: AMQPProperties,
    routing_key: Option<String>,
}

impl Dispatch {
    /// Creates a new [`DispatchBuilder`].
    pub fn builder() -> DispatchBuilder {
        DispatchBuilder::new()
    }

    /// Shorthand for creating a [`Dispatch`] with the payload set to the given
    /// bytes.
    ///
    /// This method is specifically made to take an owned `Vec<u8>`, to make sure
    /// no copying occurs and the bytes are simply moved into this dispatch.
    ///
    /// When copying of bytes is acceptable or desired, use
    /// [`from_byte_ref`](Dispatch::from_byte_ref).
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::builder().with_bytes(bytes).build()
    }

    /// Shorthand for creating a [`Dispatch`] by copying the given bytes to the
    /// payload.
    pub fn from_byte_ref(bytes: impl AsRef<[u8]>) -> Self {
        Self::builder().with_byte_ref(bytes).build()
    }

    /// Creates a new outgoing dispatch with the provided contents.
    fn new(bytes: Vec<u8>, properties: AMQPProperties, routing_key: Option<String>) -> Self {
        Self {
            bytes,
            properties,
            routing_key,
        }
    }
}

impl Dispatch {
    /// Exposes the encoded content of this message.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Exposes the properties content of this message.
    ///
    /// This getter is for internal use.
    pub(crate) fn properties(&self) -> AMQPProperties {
        self.properties.clone()
    }

    /// Exposes the routing key that is to be used just for this message.
    pub fn routing_key(&self) -> Option<&str> {
        self.routing_key.as_deref()
    }
}

/// Convenience implementations of [`From`] for [`Dispatch`].
const _: () = {
    impl From<String> for Dispatch {
        fn from(value: String) -> Self {
            Dispatch::from_bytes(value.into_bytes())
        }
    }

    impl From<&str> for Dispatch {
        fn from(value: &str) -> Self {
            Dispatch::from_byte_ref(value.as_bytes())
        }
    }

    impl From<Vec<u8>> for Dispatch {
        fn from(value: Vec<u8>) -> Self {
            Dispatch::from_bytes(value)
        }
    }

    impl From<Box<[u8]>> for Dispatch {
        fn from(value: Box<[u8]>) -> Self {
            Dispatch::from_bytes(value.into())
        }
    }

    impl From<&[u8]> for Dispatch {
        fn from(value: &[u8]) -> Self {
            Dispatch::from_byte_ref(value)
        }
    }

    impl<'a> From<Cow<'a, str>> for Dispatch {
        fn from(value: Cow<'a, str>) -> Self {
            Dispatch::from_bytes(value.into_owned().into_bytes())
        }
    }

    impl<'a> From<Cow<'a, [u8]>> for Dispatch {
        fn from(value: Cow<'a, [u8]>) -> Self {
            Dispatch::from_bytes(value.into_owned())
        }
    }
};

/// Allows building an **outgoing** RabbitMQ [`Dispatch`] iteratively.
///
/// Expects that the payload has been converted to a vector of bytes.
pub struct DispatchBuilder {
    bytes: Vec<u8>,
    properties: AMQPProperties,
    headers: FieldTable,
    routing_key: Option<String>,
}

impl DispatchBuilder {
    /// Creates a new [`Dispatch`] builder.
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            properties: AMQPProperties::default(),
            headers: FieldTable::default(),
            routing_key: None,
        }
    }

    /// Creates a builder from the given vector of bytes and the given parts of
    /// a destructed [`Envelope`].
    ///
    /// This factory is for internal use.
    pub(crate) fn from_bytes_and_properties(bytes: Vec<u8>, properties: AMQPProperties) -> Self {
        let headers: FieldTable = properties.headers().clone().unwrap_or_default();
        Self {
            bytes,
            properties,
            headers,
            routing_key: None,
        }
    }

    /// Sets the payload of this [`Dispatch`] to the given bytes.
    ///
    /// This method is specifically made to take an owned `Vec<u8>`, to make sure
    /// no copying occurs and the bytes are simply moved into this dispatch.
    ///
    /// When copying of bytes is acceptable or desired, use
    /// [`with_byte_ref`](DispatchBuilder::with_byte_ref).
    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.bytes = bytes;

        self
    }

    /// Copies the given bytes to the payload of this [`Dispatch`].
    pub fn with_byte_ref(mut self, bytes: impl AsRef<[u8]>) -> Self {
        self.bytes = bytes.as_ref().to_vec();

        self
    }

    /// Sets the durability flag in the [`AMQPProperties`] of this [`Dispatch`]
    /// to [“durable”](DeliveryMode::Durable).
    pub fn durable(mut self) -> Self {
        self.properties = self.properties.push_delivery_mode(DeliveryMode::Durable);

        self
    }

    /// Sets the durability flag in the [`AMQPProperties`] of this [`Dispatch`]
    /// to [“transient”](DeliveryMode::Transient).
    pub fn transient(mut self) -> Self {
        self.properties = self.properties.push_delivery_mode(DeliveryMode::Transient);

        self
    }

    /// Sets the priority in the [`AMQPProperties`] of this [`Dispatch`] to
    /// the given value.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.properties = self.properties.push_priority(priority);

        self
    }

    /// Sets the timestamp in the [`AMQPProperties`] of this [`Dispatch`] to
    /// the given value.
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.properties = self.properties.push_timestamp(timestamp);

        self
    }

    /// Sets the timestamp in the [`AMQPProperties`] of this [`Dispatch`] to
    /// the current timestamp.
    pub fn with_current_timestamp(mut self) -> Self {
        self.properties = self.properties.push_current_timestamp();

        self
    }

    /// Sets the attempt header of this [`Dispatch`] to the given value.
    pub fn with_attempt(mut self, attempt: u32) -> Self {
        self.headers.push_attempt(attempt);

        self
    }

    /// Increments the attempt header of this [`Dispatch`] by one. If the attempt
    /// is not yet present, it will be assumed to be zero, and will be incremented
    /// to one.
    pub fn with_incremented_attempt(mut self) -> Self {
        self.headers.increment_attempt();

        self
    }

    /// Sets the content type of this [`Dispatch`] to the given value.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder().with_content_type("application/json").build();
    /// ```
    pub fn with_content_type<T>(mut self, content_type: T) -> Self
    where
        ShortString: Morph<T>,
    {
        self.properties = self.properties.push_content_type(content_type);

        self
    }

    /// Sets the content encoding of this [`Dispatch`] to the given value.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder().with_content_encoding("gzip").build();
    /// ```
    pub fn with_content_encoding<T>(mut self, content_encoding: T) -> Self
    where
        ShortString: Morph<T>,
    {
        self.properties = self.properties.push_content_encoding(content_encoding);

        self
    }

    /// Sets a header of this [`Dispatch`] under the given `key` to the given
    /// `value`.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder()
    ///     .with_header("created_via", "strut")
    ///     .with_header("processed_in_secs", 12.9)
    ///     .build();
    /// ```
    pub fn with_header<T>(mut self, key: &str, value: T) -> Self
    where
        AMQPValue: Morph<T>,
    {
        self.headers.push(key, value);

        self
    }

    /// Sets the correlation ID of this [`Dispatch`] to the given value.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder().with_correlation_id(4489).build();
    /// ```
    pub fn with_correlation_id<T>(mut self, correlation_id: T) -> Self
    where
        ShortString: Morph<T>,
    {
        self.properties = self.properties.push_correlation_id(correlation_id);

        self
    }

    /// Sets the “reply-to” value of this [`Dispatch`] to the given value.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder().with_reply_to("me").build();
    /// ```
    pub fn with_reply_to<T>(mut self, reply_to: T) -> Self
    where
        ShortString: Morph<T>,
    {
        self.properties = self.properties.push_reply_to(reply_to);

        self
    }

    /// Sets the expiration of this [`Dispatch`] to the given value.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder().with_expiration(86400).build();
    /// ```
    pub fn with_expiration<T>(mut self, expiration: T) -> Self
    where
        ShortString: Morph<T>,
    {
        self.properties = self.properties.push_expiration(expiration);

        self
    }

    /// Sets the message ID of this [`Dispatch`] to the given value.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder().with_message_id(664833405).build();
    /// ```
    pub fn with_message_id<T>(mut self, message_id: T) -> Self
    where
        ShortString: Morph<T>,
    {
        self.properties = self.properties.push_message_id(message_id);

        self
    }

    /// Sets the kind of this [`Dispatch`] to the given value.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder().with_kind("priority").build();
    /// ```
    pub fn with_kind<T>(mut self, kind: T) -> Self
    where
        ShortString: Morph<T>,
    {
        self.properties = self.properties.push_kind(kind);

        self
    }

    /// Sets the user ID of this [`Dispatch`] to the given value.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder()
    ///     .with_user_id("01522090-f465-4a44-bd3d-9e06c061f6ac")
    ///     .build();
    /// ```
    pub fn with_user_id<T>(mut self, user_id: T) -> Self
    where
        ShortString: Morph<T>,
    {
        self.properties = self.properties.push_user_id(user_id);

        self
    }

    /// Sets the app ID of this [`Dispatch`] to the given value.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder().with_app_id("app_17").build();
    /// ```
    pub fn with_app_id<T>(mut self, app_id: T) -> Self
    where
        ShortString: Morph<T>,
    {
        self.properties = self.properties.push_app_id(app_id);

        self
    }

    /// Sets the cluster ID of this [`Dispatch`] to the given value.
    ///
    /// ## Example
    ///
    /// ```
    /// use strut_rabbitmq::Dispatch;
    ///
    /// let dispatch = Dispatch::builder().with_cluster_id(7747).build();
    /// ```
    pub fn with_cluster_id<T>(mut self, cluster_id: T) -> Self
    where
        ShortString: Morph<T>,
    {
        self.properties = self.properties.push_cluster_id(cluster_id);

        self
    }

    /// Defines a routing key to be used just for this dispatch.
    ///
    /// If this method is never called, the [`Publisher`](crate::Publisher)
    /// instead uses the routing key [configured](crate::Egress::routing_key) on
    /// the [`Egress`](crate::Egress).
    pub fn with_routing_key(mut self, routing_key: impl Into<String>) -> Self {
        self.routing_key = Some(routing_key.into());

        self
    }

    /// Builds the [`Dispatch`].
    pub fn build(self) -> Dispatch {
        // Destructure self
        let DispatchBuilder {
            bytes,
            mut properties,
            headers,
            ..
        } = self;

        // Put the headers into the properties
        properties = properties.with_headers(headers);

        Dispatch::new(bytes, properties, self.routing_key)
    }
}
