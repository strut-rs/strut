use strut_factory::Deserialize as StrutDeserialize;
use tracing_core::LevelFilter as TracingLevelFilter;

/// A thin abstraction around the `tracing` crate’s
/// [`LevelFilter`](TracingLevelFilter), introduced to provide deserialization.
///
/// A verbosity level is “higher” if it is more verbose. In this sense,
/// [`Trace`](Verbosity::Trace) is higher (more verbose) than
/// [`Error`](Verbosity::Error).
///
/// Conversely, a verbosity level is “lower” if it is less verbose. In this
/// sense, [`Warn`](Verbosity::Warn) is lower than [`Info`](Verbosity::Info).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub enum Verbosity {
    /// Log **nothing**.
    #[strut(alias = "no")]
    Off,

    /// Log at level [`ERROR`](tracing_core::metadata::Level::ERROR) only.
    #[strut(alias = "err")]
    Error,

    /// Log at level [`WARN`](tracing_core::metadata::Level::WARN) and lower.
    #[strut(alias = "warning")]
    Warn,

    /// Log at level [`INFO`](tracing_core::metadata::Level::INFO) and lower.
    Info,

    /// Log at level [`DEBUG`](tracing_core::metadata::Level::DEBUG) and lower.
    Debug,

    /// Log **everything**.
    Trace,
}

impl Default for Verbosity {
    /// Defines a reasonable default [`Verbosity`].
    fn default() -> Self {
        Self::Info
    }
}

impl Verbosity {
    /// Translates this [`Verbosity`] level to the `tracing` crate’s
    /// [`LevelFilter`](TracingLevel).
    pub fn to_tracing_level_filter(&self) -> TracingLevelFilter {
        match self {
            Self::Off => TracingLevelFilter::OFF,
            Self::Error => TracingLevelFilter::ERROR,
            Self::Warn => TracingLevelFilter::WARN,
            Self::Info => TracingLevelFilter::INFO,
            Self::Debug => TracingLevelFilter::DEBUG,
            Self::Trace => TracingLevelFilter::TRACE,
        }
    }
}

impl From<Verbosity> for TracingLevelFilter {
    fn from(value: Verbosity) -> Self {
        value.to_tracing_level_filter()
    }
}

impl From<&Verbosity> for TracingLevelFilter {
    fn from(value: &Verbosity) -> Self {
        value.to_tracing_level_filter()
    }
}
