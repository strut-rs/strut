use strut_factory::Deserialize as StrutDeserialize;

pub mod exchange;
pub mod header;
pub mod queue;

/// Defines how the consumed messages are acknowledged: explicitly or implicitly.
///
/// If transactional message handling is required, it is highly recommended to
/// use the [manual](AckingBehavior::Manual) mode, as otherwise it is quite
/// possible to lose messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub enum AckingBehavior {
    /// Messages must be explicitly acknowledged by the application logic.
    ///
    /// This is the recommended mode for transactional message handling: a
    /// message will not be removed from the queue until its [`Envelope`] is
    /// explicitly finalized. This, however, means that some messages may end up
    /// being processed more than once. Thus, this mode provides the
    /// at-least-once message processing semantics.
    Manual,

    /// Messages are implicitly pre-acknowledged by the server on delivery,
    /// before even reaching the application logic.
    ///
    /// **This mode will cause missed messages** on consumption! Some messages
    /// may occasionally fall through the cracks before being handed to the
    /// consuming logic, and since the messages are implicitly pre-acknowledged
    /// — such messages will not be re-delivered or seen again.
    ///
    /// If it is important to process each message no more than once, but
    /// occasional message loss is acceptable — this is the mode of choice.
    ///
    /// For transactional handling with at-least-once semantics, use the
    /// [manual](AckingBehavior::Manual) mode.
    ///
    /// This mode effectively disables the [prefetch](Ingress::prefetch_count).
    /// The prefetch count refers to the count of **unacknowledged** messages
    /// pre-delivered to a consumer, and with implicit auto-acking there are no
    /// unacknowledged messages at all.
    Auto,
}

/// Defines the matching behavior for the
/// [`Headers`](ExchangeKind::Headers) kind of exchange. Irrelevant for
/// all other kinds of exchanges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub enum HeadersMatchingBehavior {
    /// All headers must match (`x-match: all`).
    All,

    /// At least one header must match (`x-match: any`).
    Any,
}

impl HeadersMatchingBehavior {
    /// Returns the appropriate string header value recognized by RabbitMQ.
    pub const fn rabbitmq_value(&self) -> &str {
        match self {
            HeadersMatchingBehavior::All => "all",
            HeadersMatchingBehavior::Any => "any",
        }
    }
}
