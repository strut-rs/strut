#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

/// Exposes an application configuration section.
mod config;
pub use self::config::RabbitMqConfig;

/// Exposes a handle for defining a set of connection credentials.
mod handle;
pub use self::handle::{DsnChunks, Handle, HandleCollection};

/// Exposes various types for defining outbound and inbound message routes.
mod routing {
    pub mod egress;
    pub mod exchange;
    pub mod ingress;
}

// Re-export routing types
pub use self::routing::egress::{Egress, EgressBuilder, EgressError, EgressLandscape};
pub use self::routing::exchange::{CustomExchange, Exchange, ExchangeBuilder, ExchangeError};
pub use self::routing::ingress::queue::Queue;
pub use self::routing::ingress::{Ingress, IngressBuilder, IngressError, IngressLandscape};

/// Exposes machinery for maintaining a connection to a RabbitMQ cluster.
mod connector;
pub use self::connector::{Connector, Gateway};

/// Exposes machinery for transporting incoming and outgoing messages.
mod transport {
    pub mod inbound;
    pub mod outbound;
}

// Re-export inbound types
pub use self::transport::inbound::decoder::{Decoder, NoopDecoder, StringDecoder};
pub use self::transport::inbound::envelope::{stack::EnvelopeStack, Envelope};
pub use self::transport::inbound::subscriber::{
    DeclarationError, StringSubscriber, Subscriber, UndecodedSubscriber,
};

// Re-export inbound types (JSON-related)
#[cfg(feature = "json")]
pub use self::transport::inbound::decoder::JsonDecoder;
#[cfg(feature = "json")]
pub use self::transport::inbound::subscriber::JsonSubscriber;

// Re-export outbound types
pub use self::transport::outbound::dispatch::{Dispatch, DispatchBuilder};
pub use self::transport::outbound::publisher::api::{
    BatchPublishingError, BatchPublishingResult, PublishingError, PublishingFailure,
    PublishingResult,
};
pub use self::transport::outbound::publisher::Publisher;

// Re-export [`NonEmpty`] as it is part of this crate’s API.
pub use nonempty::NonEmpty;

/// Exposes convenience layers around `lapin` types.
pub mod util {
    mod amqp_properties;
    pub use self::amqp_properties::push::{
        PushAppId, PushClusterId, PushContentEncoding, PushContentType, PushCorrelationId,
        PushExpiration, PushHeader, PushKind, PushMessageId, PushReplyTo, PushUserId,
    };
    pub use self::amqp_properties::retrieve::{
        RetrieveAppId, RetrieveClusterId, RetrieveContentEncoding, RetrieveContentType,
        RetrieveCorrelationId, RetrieveExpiration, RetrieveHeader, RetrieveKind, RetrieveMessageId,
        RetrieveReplyTo, RetrieveUserId,
    };
    pub use self::amqp_properties::RetrievePushMap;

    mod amqp_value;
    pub use self::amqp_value::IsEmpty;

    mod coerce;
    pub use self::coerce::Coerce;

    mod field_table;
    pub use self::field_table::push::Push;
    pub use self::field_table::retrieve::Retrieve;
    pub use self::field_table::{Attempt, HEADER_ATTEMPT};

    mod morph;
    pub use self::morph::Morph;
}

/// Exposes the domain enumerations that are used to encode the underlying
/// configuration and routing logic.
mod repr {
    pub mod delivery;
    pub mod egress;
    pub mod ingress;
}
pub use self::repr::delivery::{DeliveryMode, FinalizationKind};
pub use self::repr::egress::ConfirmationLevel;
pub use self::repr::ingress::exchange::{
    ExchangeKind, EXCHANGE_AMQ_DIRECT, EXCHANGE_AMQ_FANOUT, EXCHANGE_AMQ_HEADERS,
    EXCHANGE_AMQ_MATCH, EXCHANGE_AMQ_TOPIC, EXCHANGE_DEFAULT,
};
pub use self::repr::ingress::header::Header;
pub use self::repr::ingress::queue::{QueueKind, QueueRenamingBehavior};
pub use self::repr::ingress::{AckingBehavior, HeadersMatchingBehavior};

/// Re-exports the `strut_shutdown` function to facilitate stand-alone usage of
/// this crate.
///
/// When using this crate without the `strut` framework itself, await on this
/// function as a last thing before completing the main application logic.
pub use strut_core::strut_shutdown;
