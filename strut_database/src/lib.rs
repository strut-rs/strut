#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

// Compile time check to ensure no more than one default database type is active
#[cfg(any(
    all(
        feature = "default-mysql",
        any(feature = "default-postgres", feature = "default-sqlite"),
    ),
    all(
        feature = "default-postgres",
        any(feature = "default-mysql", feature = "default-sqlite"),
    ),
    all(
        feature = "default-sqlite",
        any(feature = "default-mysql", feature = "default-postgres"),
    ),
))]
compile_error!(
    "No more than one of the following feature flags may be enabled at a time: `default-mysql`, `default-postgres`, `default-sqlite`."
);

/// Exposes the data structures used in the configuration section.
mod repr {
    pub mod cert;
    pub mod handle;
    pub mod log;
    pub mod pool;
}
#[cfg(feature = "mysql")]
pub use self::repr::handle::mysql::{MySqlHandle, MySqlHandleCollection};
#[cfg(feature = "postgres")]
pub use self::repr::handle::postgres::{PostgresHandle, PostgresHandleCollection};
#[cfg(feature = "sqlite")]
pub use self::repr::handle::sqlite::{SqliteHandle, SqliteHandleCollection};

/// Exposes an application configuration section.
mod config;
pub use self::config::DatabaseConfig;

/// Exposes machinery for maintaining a pool of connections.
mod connector;
pub use self::connector::Connector;

/// Implements a migrations worker.
mod migrations;
pub use self::migrations::MigrationsWorker;
pub use strut_sync::Gate;

/// Re-exports the public API of `sqlx` for convenience.
pub use sqlx;


/// Re-exports the `strut_shutdown` function to facilitate stand-alone usage of
/// this crate.
///
/// When using this crate without the `strut` crate itself, await on this
/// function as a last thing before completing the main application logic.
pub use strut_core::strut_shutdown;
