use strut_factory::Deserialize as StrutDeserialize;

/// Defines the extent to which the message [`Publisher`](crate::Publisher)
/// should confirm successful sending.
///
/// If the confirmation level is set to the
/// [lowest level](ConfirmationLevel::Transmitted), then the confirmation of the
/// message is going-to be a no-op, without any network communication. If,
/// however, the confirmation level is anywhere higher, the confirmation is
/// performed against the RabbitMQ broker asynchronously, and the publishing
/// implicitly switches to at-least-once publishing guarantee, which means that
/// some of the messages may be published multiple times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub enum ConfirmationLevel {
    /// Ensures network transmission.
    #[strut(alias = "transmit")]
    Transmitted,

    /// Ensures network transmission **and** exchange existence.
    #[strut(alias = "accept")]
    Accepted,

    /// Ensures network transmission **and** exchange existence **and** routing to a queue.
    #[strut(alias = "route")]
    Routed,
}
