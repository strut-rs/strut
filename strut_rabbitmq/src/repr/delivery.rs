use crate::transport::inbound::delivery::{abandon_delivery, backwash_delivery, complete_delivery};
use lapin::acker::Acker;
use lapin::types::ShortShortUInt;
use strut_factory::Deserialize as StrutDeserialize;

/// Defines whether RabbitMQ persists the messages to disk, which affects
/// whether such messages are able to survive a broker restart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub enum DeliveryMode {
    /// Delivery mode `1`: non-persistent (transient): messages sent with this
    /// mode will **not** survive a broker restart.
    Transient,
    /// Delivery mode `2`: persistent (durable): messages sent with this mode
    /// will be written to disk and, if they are **also** routed to a **durable
    /// queue**, they **will** survive a broker restart.
    Durable,
}

/// Represents the supporting ways of finalizing an incoming RabbitMQ message.
/// Relevant only when the inbound messages are set to be
/// [manually](crate::AckingBehavior::Manual) acknowledged on the
/// [`Ingress`](crate::Ingress).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub enum FinalizationKind {
    /// Positively acknowledge the message.
    Complete,
    /// Reject (negatively acknowledge) the message and re-queue it.
    Backwash,
    /// Reject (negatively acknowledge) the message without re-queueing.
    Abandon,
}

impl DeliveryMode {
    /// Returns the appropriate `u8` value recognized by RabbitMQ.
    pub const fn rabbitmq_value(&self) -> u8 {
        match self {
            DeliveryMode::Transient => 1,
            DeliveryMode::Durable => 2,
        }
    }
}

impl From<ShortShortUInt> for DeliveryMode {
    fn from(value: ShortShortUInt) -> Self {
        match value {
            2 => DeliveryMode::Durable,
            _ => DeliveryMode::Transient,
        }
    }
}

impl FinalizationKind {
    pub(crate) async fn apply(self, subscriber: &str, acker: &Acker, bytes: &[u8]) {
        match self {
            FinalizationKind::Complete => complete_delivery(subscriber, &acker, &bytes).await,
            FinalizationKind::Backwash => backwash_delivery(subscriber, &acker, &bytes).await,
            FinalizationKind::Abandon => abandon_delivery(subscriber, &acker, &bytes).await,
        };
    }
}
