#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

/// Exposes an application configuration section.
mod config;
pub use self::config::SentryConfig;

/// Implements initialization logic for Sentry integration.
mod integration;
pub use self::integration::SentryIntegration;
pub use sentry::ClientInitGuard as SentryGuard;

/// Implements a customized [`SentryLayer`](sentry_tracing::SentryLayer) that
/// can be integrated into the [`Subscriber`](tracing::Subscriber) for setting
/// up the [`tracing`](::tracing) crate.
#[cfg(feature = "tracing")]
pub mod tracing;

/// Re-exports the `strut_shutdown` function to facilitate stand-alone usage of
/// this crate.
///
/// When using this crate without the `strut` crate itself, await on this
/// function as a last thing before completing the main application logic.
pub use strut_core::strut_shutdown;
