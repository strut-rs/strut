use crate::transport::inbound::delivery::{abandon_delivery, backwash_delivery, complete_delivery};
use crate::util::{
    Coerce, RetrieveAppId, RetrieveClusterId, RetrieveContentEncoding, RetrieveContentType,
    RetrieveCorrelationId, RetrieveExpiration, RetrieveHeader, RetrieveKind, RetrieveMessageId,
    RetrievePushMap, RetrieveReplyTo, RetrieveUserId,
};
use crate::{Decoder, DeliveryMode, DispatchBuilder};
use lapin::acker::Acker;
use lapin::message::Delivery;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::{AMQPValue, ShortString};
use parking_lot::Mutex as SyncMutex;
use std::sync::Arc;
use tracing::error;

pub mod stack;

/// Represents an **incoming** RabbitMQ message.
///
/// This class owns both the bytes of the original message’s payload and the
/// decoded (e.g., deserialized) payload `T`, along with the implementation
/// details of the original [`Delivery`].
#[derive(Debug)]
pub struct Envelope<T> {
    /// The name of the subscriber that received this message.
    subscriber: Arc<str>,
    /// The original delivery tag.
    delivery_tag: u64,
    /// The original target exchange used to send the message.
    exchange: ShortString,
    /// The original routing key used to send the message.
    routing_key: ShortString,
    /// The original redelivery flag.
    is_redelivered: bool,
    /// The original properties.
    properties: AMQPProperties,
    /// The original bytes.
    bytes: Vec<u8>,
    /// The acker associated with this message (optional for cases where messages
    /// are acked [automatically](crate::AckingBehavior::Auto)).
    acker: SyncMutex<Option<Acker>>,
    /// The decoded content of the underlying message, stored alongside its
    /// original bytes.
    payload: T,
}

/// Represents a failed attempt to create an [`Envelope`] from a [`Decoder`] and
/// a [`Delivery`].
pub(crate) struct DecoderError<D>
where
    D: Decoder,
{
    /// The original bytes that were not decoded.
    pub(crate) bytes: Vec<u8>,
    /// The acker of the original message.
    pub(crate) acker: Option<Acker>,
    /// The decoder error.
    pub(crate) error: D::Error,
}

impl<T> Envelope<T> {
    /// Attempts to create an envelope from the given [`Delivery`] using the
    /// provided [`Decoder`] implementation for interpreting the message payload.
    ///
    /// The given `is_pending` flag indicates whether this message is not yet
    /// finalized and thus, whether the [`Acker`] should be extracted and used.
    pub(crate) fn try_from<D>(
        subscriber: Arc<str>,
        decoder: &D,
        delivery: Delivery,
        is_pending: bool,
    ) -> Result<Envelope<T>, DecoderError<D>>
    where
        D: Decoder<Result = T>,
    {
        // Destructure inputs
        let Delivery {
            delivery_tag,
            exchange,
            routing_key,
            redelivered: is_redelivered,
            properties,
            data: bytes,
            acker,
        } = delivery;
        let acker = is_pending.then_some(acker);

        // Attempt to decode the given bytes with the given decoder
        match decoder.decode(&bytes) {
            // Successfully decoded
            Ok(payload) => Ok(Self {
                subscriber,
                delivery_tag,
                exchange,
                routing_key,
                is_redelivered,
                bytes,
                properties,
                acker: SyncMutex::new(acker),
                payload,
            }),

            // Failed to decode
            Err(error) => Err(DecoderError {
                bytes,
                acker,
                error,
            }),
        }
    }
}

impl<T> Envelope<T> {
    /// Exposes the delivery tag of the underlying incoming message.
    pub fn delivery_tag(&self) -> u64 {
        self.delivery_tag
    }

    /// Exposes the original target exchange used to send the underlying
    /// incoming message.
    pub fn exchange(&self) -> &str {
        self.exchange.as_str()
    }

    /// Exposes the original routing key used to send the underlying incoming
    /// message.
    pub fn routing_key(&self) -> &str {
        self.routing_key.as_str()
    }

    /// Exposes the original redelivery flag of the underlying incoming message.
    ///
    /// A message is redelivered when it has been previously dropped without
    /// [finalizing](FinalizationKind), or if it was previously explicitly
    /// [backwashed](FinalizationKind::Backwash).
    pub fn is_redelivered(&self) -> bool {
        self.is_redelivered
    }

    /// Exposes the original bytes of this message.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Exposes the decoded payload of this message.
    ///
    /// Most [decoders](Decoder) derive the payload from the original
    /// [bytes](Envelope::bytes) of this message. A notable exceptions is the
    /// [`NoopDecoder`](crate::NoopDecoder).
    pub fn payload(&self) -> &T {
        &self.payload
    }

    /// Exposes the attempt number of the underlying incoming message, if present.
    pub fn attempt(&self) -> Option<u32> {
        self.properties.retrieve_attempt()
    }

    /// Exposes the delivery mode of the underlying incoming message, if present.
    pub fn delivery_mode(&self) -> Option<DeliveryMode> {
        self.properties.retrieve_delivery_mode()
    }

    /// Exposes the priority of the underlying incoming message, if present.
    pub fn priority(&self) -> Option<u8> {
        self.properties.retrieve_priority()
    }

    /// Exposes the timestamp of the underlying incoming message, if present.
    pub fn timestamp(&self) -> Option<u64> {
        self.properties.retrieve_timestamp()
    }
}

impl<T> Envelope<T> {
    /// Reports the content type of this message, if present, [coercing](Coerce)
    /// it into type `R`, if supported.
    pub fn content_type<'a, R>(&'a self) -> Option<R>
    where
        ShortString: Coerce<'a, R>,
    {
        self.properties.retrieve_content_type()
    }

    /// Reports the content encoding of this message, if present,
    /// [coercing](Coerce) it into type `R`, if supported.
    pub fn content_encoding<'a, R>(&'a self) -> Option<R>
    where
        ShortString: Coerce<'a, R>,
    {
        self.properties.retrieve_content_encoding()
    }

    /// Reports the header value from this message by key, if present,
    /// [coercing](Coerce) it into type `R`, if supported.
    pub fn header<'a, R>(&'a self, key: &str) -> Option<R>
    where
        AMQPValue: Coerce<'a, R>,
    {
        self.properties.retrieve_header(key)
    }

    /// Reports the correlation ID of this message, if present,
    /// [coercing](Coerce) it into type `R`, if supported.
    pub fn correlation_id<'a, R>(&'a self) -> Option<R>
    where
        ShortString: Coerce<'a, R>,
    {
        self.properties.retrieve_correlation_id()
    }

    /// Reports the “reply-to” value of this message, if present,
    /// [coercing](Coerce) it into type `R`, if supported.
    pub fn reply_to<'a, R>(&'a self) -> Option<R>
    where
        ShortString: Coerce<'a, R>,
    {
        self.properties.retrieve_reply_to()
    }

    /// Reports the expiration of this message, if present, [coercing](Coerce)
    /// it into type `R`, if supported.
    pub fn expiration<'a, R>(&'a self) -> Option<R>
    where
        ShortString: Coerce<'a, R>,
    {
        self.properties.retrieve_expiration()
    }

    /// Reports the ID of this message, if present, [coercing](Coerce) it into
    /// type `R`, if supported.
    pub fn message_id<'a, R>(&'a self) -> Option<R>
    where
        ShortString: Coerce<'a, R>,
    {
        self.properties.retrieve_message_id()
    }

    /// Reports the kind of this message, if present, [coercing](Coerce) it into
    /// type `R`, if supported.
    pub fn kind<'a, R>(&'a self) -> Option<R>
    where
        ShortString: Coerce<'a, R>,
    {
        self.properties.retrieve_kind()
    }

    /// Reports the user ID of this message, if present, [coercing](Coerce) it
    /// into type `R`, if supported.
    pub fn user_id<'a, R>(&'a self) -> Option<R>
    where
        ShortString: Coerce<'a, R>,
    {
        self.properties.retrieve_user_id()
    }

    /// Reports the app ID of this message, if present, [coercing](Coerce) it
    /// into type `R`, if supported.
    pub fn app_id<'a, R>(&'a self) -> Option<R>
    where
        ShortString: Coerce<'a, R>,
    {
        self.properties.retrieve_app_id()
    }

    /// Reports the cluster ID of this message, if present, [coercing](Coerce)
    /// it into type `R`, if supported.
    pub fn cluster_id<'a, R>(&'a self) -> Option<R>
    where
        ShortString: Coerce<'a, R>,
    {
        self.properties.retrieve_cluster_id()
    }
}

impl<T> Envelope<T> {
    /// Copies the bytes of this envelope into a new [`DispatchBuilder`], which
    /// will represent a message almost identical to the one that created this
    /// initial envelope.
    ///
    /// The intention of this method is to enable sending an incoming envelope
    /// back to RabbitMQ, unchanged, e.g., for the purpose of retrying the
    /// processing of the message.
    pub fn dispatch_builder(&self) -> DispatchBuilder {
        // Clone main building blocks
        let bytes = self.bytes.clone();
        let properties = self.properties.clone();

        DispatchBuilder::from_bytes_and_properties(bytes, properties)
    }

    /// [Completes](FinalizationKind::Complete) the incoming message represented
    /// by this [`Envelope`]. If this envelope has already been
    /// [finalized](FinalizationKind) (either
    /// [automatically](crate::AckingBehavior::Auto), or manually via a method
    /// like this one), this method is a no-op.
    pub async fn complete(self) {
        let optional_acker = self.acker.lock().take();
        if let Some(ref acker) = optional_acker {
            complete_delivery(self.subscriber.as_ref(), acker, &self.bytes).await;
        }
    }

    /// [Backwashes](FinalizationKind::Backwash) the incoming message
    /// represented by this [`Envelope`]. If this envelope has already been
    /// [finalized](FinalizationKind) (either
    /// [automatically](crate::AckingBehavior::Auto), or manually via a method
    /// like this one), this method is a no-op.
    pub async fn backwash(self) {
        let optional_acker = self.acker.lock().take();
        if let Some(ref acker) = optional_acker {
            backwash_delivery(self.subscriber.as_ref(), acker, &self.bytes).await;
        }
    }

    /// [Abandons](FinalizationKind::Abandon) the incoming message represented
    /// by this [`Envelope`]. If this envelope has already been
    /// [finalized](FinalizationKind) (either
    /// [automatically](crate::AckingBehavior::Auto), or manually via a method
    /// like this one), this method is a no-op.
    pub async fn abandon(self) {
        let optional_acker = self.acker.lock().take();
        if let Some(ref acker) = optional_acker {
            abandon_delivery(self.subscriber.as_ref(), acker, &self.bytes).await;
        }
    }
}

impl<T> Envelope<T>
where
    T: Default,
{
    /// Consumes this envelope and returns its building blocks for manual
    /// handling. This allows to effectively opt out of this crate’s handling
    /// mechanisms.
    ///
    /// This function is being considered for the public API. The signature of
    /// this method is intentionally clunky, as it is not intended for widespread
    /// use.
    ///
    /// Because [`Envelope`] implements [`Drop`], it cannot be simply destructured.
    /// We have to [`take`](std::mem::take) every element out of it, and this means
    /// that every element (including `T`) must implement [`Default`]. The caller
    /// must anticipate that a new, default instance of `T` will be constructed
    /// in this method call.
    #[allow(dead_code)]
    pub(crate) fn destruct(mut self) -> (Vec<u8>, AMQPProperties, Option<Acker>, T) {
        // Pick `self` apart
        let bytes = std::mem::take(&mut self.bytes);
        let properties = std::mem::take(&mut self.properties);
        let acker = self.acker.lock().take();
        let content = std::mem::take(&mut self.payload);

        // Return the parts
        (bytes, properties, acker, content)
    }
}

#[cfg(test)]
impl Envelope<()> {
    /// Creates a new instance with given [`AMQPProperties`].
    pub fn test_dud(properties: AMQPProperties) -> Self {
        use crate::util::Morph;

        Self {
            subscriber: Arc::from("test"),
            delivery_tag: 0,
            exchange: ShortString::morph(""),
            routing_key: ShortString::morph(""),
            is_redelivered: false,
            properties,
            bytes: vec![],
            acker: SyncMutex::new(None),
            payload: (),
        }
    }
}

impl<T> Drop for Envelope<T> {
    fn drop(&mut self) {
        if self.acker.lock().is_some() {
            error!(
                alert = true,
                byte_preview = String::from_utf8_lossy(&self.bytes).as_ref(),
                "Dropped an envelope without finalizing it",
            );
        }
    }
}
