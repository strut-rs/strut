use crate::LifetimeId;
use std::sync::OnceLock;

mod index;
pub mod lifetime_id;

/// Exposes the ways for the application to introspect its own runtime
/// replica, primarily via a [numerical index](AppReplica::index) (injected via
/// the environment at runtime).
pub struct AppReplica;

impl AppReplica {
    /// Returns the `usize` index of this application’s replica. Lazily discerns
    /// the value on the first call, then returns a copy of the same value on
    /// each repeated call.
    ///
    /// This value can be set from the `APP_REPLICA_INDEX` environment variable
    /// at **runtime**. The same environment variable set at compile time
    /// produces no effect.
    ///
    /// If the environment variable is not set at runtime, or its value is not a
    /// valid `usize` integer, this method will return [`None`]. Otherwise, no
    /// validation is performed on the value.
    pub fn index() -> Option<usize> {
        static INDEX: OnceLock<Option<usize>> = OnceLock::new();

        INDEX.get_or_init(index::discern).clone()
    }

    /// Returns a pseudo-randomized [`LifetimeId`] of this application’s replica
    /// that is stable throughout a single runtime. Lazily generates the value
    /// on the first call.
    ///
    /// This pseudo-randomized value is **not** cryptographically secure.
    pub fn lifetime_id() -> &'static LifetimeId {
        static LIFETIME_ID: OnceLock<LifetimeId> = OnceLock::new();

        LIFETIME_ID.get_or_init(LifetimeId::random)
    }
}
