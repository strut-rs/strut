use strut_factory::Deserialize as StrutDeserialize;

/// Defines a RabbitMQ queue kind, which is currently limited to either **classic**
/// or **quorum** queues. See the RabbitMQ documentation for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub enum QueueKind {
    /// Classic queues are stored on a single node in the RabbitMQ cluster and
    /// provide high throughput.
    ///
    /// A classic queue may still be mirrored to other nodes, which provides a degree
    /// of high availability.
    Classic,
    /// Quorum queues are stored on multiple nodes in the RabbitMQ cluster and
    /// provide high availability using a quorum algorithm.
    Quorum,
}

/// Defines optional queue renaming behavior. Values other than
/// [`Verbatim`](QueueRenamingBehavior::Verbatim) trigger appending of a
/// suffix to the user-provided queue name.
///
/// When a suffix is added, it is separated from the preceding queue name with a
/// full-stop `.` character. A suffix is never added to an empty queue name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub enum QueueRenamingBehavior {
    /// Leaves the queue name unchanged (doesn’t add any suffix).
    #[strut(alias = "none")]
    Verbatim,

    /// Suffixes the queue name with the application replica index (as reported
    /// by [`AppReplica::index`](strut_core::AppReplica::index)).
    #[strut(alias = "index")]
    ReplicaIndex,

    /// Suffixes the queue name with the application replica lifetime ID (as reported
    /// by [`AppReplica::lifetime_id`](strut_core::AppReplica::lifetime_id)).
    #[strut(alias = "lifetime", alias = "lifetime_id")]
    ReplicaLifetimeId,
}

impl QueueKind {
    /// Returns the appropriate string header value recognized by RabbitMQ.
    pub const fn rabbitmq_value(&self) -> &str {
        match self {
            QueueKind::Classic => "classic",
            QueueKind::Quorum => "quorum",
        }
    }
}
