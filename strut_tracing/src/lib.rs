#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

/// Implements the [`TracingConfig`] application configuration section.
mod config;
pub use self::config::flavor::FormatFlavor;
pub use self::config::verbosity::Verbosity;
pub use self::config::TracingConfig;

/// Implements the custom formatted `tracing` layer.
mod fmt;
pub use self::fmt::make_layer;

/// Partly re-exports the public API of `tracing_*` for convenience.
pub use tracing_core::Subscriber;
pub use tracing_subscriber::layer::SubscriberExt;
pub use tracing_subscriber::util::SubscriberInitExt;
pub use tracing_subscriber::Registry;
