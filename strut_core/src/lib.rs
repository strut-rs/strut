#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

/// Application profile.
mod profile;
pub use self::profile::AppProfile;

/// Application context.
mod context;
pub use self::context::AppContext;

/// Application replica facade.
mod replica;
pub use self::replica::lifetime_id::{Glued, Hyphenated, LifetimeId, Underscored};
pub use self::replica::AppReplica;

/// Application spindown registry & tokens.
mod spindown;
pub use self::spindown::{token::AppSpindownToken, AppSpindown};

/// Implements a [`Pivot`] facade for centralized resolution of the pivot directory
mod pivot;
pub use self::pivot::Pivot;

/// Globally recognized field name that, when present in a `tracing` macro call,
/// should trigger an event for an external alerting system.
pub const ALERT_FIELD_NAME: &str = "alert";

/// [Terminates](AppContext::terminate) the global [`AppContext`] and waits for
/// [`AppSpindown`] to complete.
///
/// This is effectively the global shutdown&clean-up routine for all workloads
/// that integrate with the Strut family of crates via [`AppContext`] and
/// [`AppSpindown`].
///
/// ## Usage
///
/// When using any of the public `strut` components without the framework
/// itself, await on this function as a last thing before completing the main
/// application logic.
pub async fn strut_shutdown() {
    // Terminate the global application context
    AppContext::terminate();

    // Wait for the registered spindown workloads to finish
    AppSpindown::completed().await;
}
