use strut_factory::Deserialize as StrutDeserialize;

/// Represents a particular preset of configuration for the
/// [event formatter](tracing_subscriber::fmt::format::Format) used by the
/// [formatted `Subscriber`](tracing_subscriber::fmt::Subscriber) of the
/// `tracing_subscriber` crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub enum FormatFlavor {
    /// Uses the default [`Full`](tracing_subscriber::fmt::format::Full) event formatting.
    Full,

    /// Uses the [`Compact`](tracing_subscriber::fmt::format::Compact) event formatting.
    Compact,

    /// Uses the multi-line [`Pretty`](tracing_subscriber::fmt::format::Pretty) event formatting.
    Pretty,

    #[cfg(feature = "json")]
    /// Uses the [`Json`](tracing_subscriber::fmt::format::Json) event formatting.
    Json,
}

impl Default for FormatFlavor {
    /// Defines a reasonable default [`FormatFlavor`].
    fn default() -> Self {
        FormatFlavor::Full
    }
}
