#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

/// Implements an opinionated version of the exponential backoff.
#[cfg(feature = "backoff")]
mod backoff {
    pub mod config;
    pub mod wrapper;
}
#[cfg(feature = "backoff")]
pub use self::backoff::{config::BackoffConfig, wrapper::Backoff};
