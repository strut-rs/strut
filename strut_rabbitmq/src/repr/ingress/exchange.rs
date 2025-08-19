use lapin::ExchangeKind as LapinExchangeKind;
use std::fmt::{Display, Formatter};
use strut_factory::Deserialize as StrutDeserialize;

/// Name of the RabbitMQ built-in default exchange
pub const EXCHANGE_DEFAULT: &str = "";

/// Name of the RabbitMQ built-in `amq.direct` exchange
pub const EXCHANGE_AMQ_DIRECT: &str = "amq.direct";

/// Name of the RabbitMQ built-in `amq.fanout` exchange
pub const EXCHANGE_AMQ_FANOUT: &str = "amq.fanout";

/// Name of the RabbitMQ built-in `amq.headers` exchange
pub const EXCHANGE_AMQ_HEADERS: &str = "amq.headers";

/// Name of the RabbitMQ built-in `amq.match` exchange
pub const EXCHANGE_AMQ_MATCH: &str = "amq.match";

/// Name of the RabbitMQ built-in `amq.topic` exchange
pub const EXCHANGE_AMQ_TOPIC: &str = "amq.topic";

/// Represents the supported kinds of RabbitMQ exchanges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub enum ExchangeKind {
    /// A **direct** exchange delivers messages to queues based on the message
    /// routing key.
    Direct,

    /// A **fanout** exchange routes messages to all the queues that are bound
    /// to it, and the routing key is ignored.
    #[strut(alias = "fan")]
    Fanout,

    /// A **headers** exchange is designed for routing on multiple attributes
    /// that are more easily expressed as message headers than a routing key.
    #[strut(alias = "header")]
    Headers,

    /// **Topic** exchanges route messages to one or many queues based on matching
    /// between a message routing key and the pattern that was used to bind a
    /// queue to an exchange.
    Topic,

    /// The **consistent-hash** exchange type uses consistent hashing to distribute
    /// messages between the bound queues.
    ///
    /// This subtype applies the hashing to the **routing key** on the message
    /// (not the routing key used when binding a queue to an exchange).
    #[strut(
        alias = "hash_keys",
        alias = "hash_routing_key",
        alias = "hash_routing_keys"
    )]
    HashKey,

    /// The **consistent-hash** exchange type uses consistent hashing to distribute
    /// messages between the bound queues.
    ///
    /// This subtype applies the hashing to the **message ID**.
    #[strut(
        alias = "hash_ids",
        alias = "hash_message_id",
        alias = "hash_message_ids"
    )]
    HashId,
}

impl ExchangeKind {
    /// Returns the [`lapin::ExchangeKind`] value corresponding to this exchange
    /// kind.
    pub fn lapin_value(&self) -> LapinExchangeKind {
        match self {
            Self::Direct => LapinExchangeKind::Direct,
            Self::Fanout => LapinExchangeKind::Fanout,
            Self::Headers => LapinExchangeKind::Headers,
            Self::Topic => LapinExchangeKind::Topic,
            Self::HashKey | Self::HashId => LapinExchangeKind::Custom("x-consistent-hash".into()),
        }
    }
}

impl Display for ExchangeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ExchangeKind::Direct => "direct",
            ExchangeKind::Fanout => "fanout",
            ExchangeKind::Headers => "headers",
            ExchangeKind::Topic => "topic",
            ExchangeKind::HashKey => "x-consistent-hash",
            ExchangeKind::HashId => "x-consistent-hash",
        })
    }
}
