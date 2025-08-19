#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

/// Implements component-specific facades.
mod facade {
    /// Implements the application configuration facades.
    pub mod config {
        /// Implements the [`AppConfig`] facade.
        pub mod initial;

        /// Implements the [`AppLiveConfig`] facade.
        #[cfg(feature = "config-live")]
        pub mod live;

        /// Implements the custom [`AppConfigError`] type.
        pub mod error;
    }

    /// Implements the [`DotEnv`] facade.
    pub mod dotenv;

    /// Implements the [`Database`] facade.
    #[cfg(any(
        feature = "database-mysql",
        feature = "database-postgres",
        feature = "database-sqlite",
    ))]
    pub mod database;

    /// Implements the [`RabbitMq`] facade.
    #[cfg(feature = "rabbitmq")]
    pub mod rabbitmq;
}

/// Re-exports the [`AppConfig`]-related types.
pub use self::facade::config::error::AppConfigError;
pub use self::facade::config::initial::AppConfig;
#[cfg(feature = "config-live")]
pub use self::facade::config::live::AppLiveConfig;


/// Re-exports the [`DotEnv`] facade.
pub use self::facade::dotenv::DotEnv;


/// Re-exports the [`Database`] facade.
#[cfg(any(
    feature = "database-mysql",
    feature = "database-postgres",
    feature = "database-sqlite",
))]
pub use self::facade::database::Database;


/// Re-exports the [`RabbitMq`] facade.
#[cfg(feature = "rabbitmq")]
pub use self::facade::rabbitmq::RabbitMq;


/// Re-exports the public API of `strut-core` in the root of this crate for
/// convenience.
pub use strut_core::*;


/// Re-exports the public API of `tokio` for convenience.
pub use tokio;


/// Re-exports the public API of `strut-config` for convenience.
pub use strut_config as config;


/// Partly re-exports the public API of `tracing` for convenience.
#[cfg(feature = "tracing")]
pub use tracing;


/// Re-exports the public API of `strut-database` for convenience.
#[cfg(any(
    feature = "database-mysql",
    feature = "database-postgres",
    feature = "database-sqlite",
))]
pub use strut_database as database;


/// Re-exports the public API of `strut-rabbitmq` for convenience.
#[cfg(feature = "rabbitmq")]
pub use strut_rabbitmq as rabbitmq;


/// Implements the [`Launchpad`] utility for building an [`App`].
mod launchpad;
pub use self::launchpad::wiring::configuration::ConfigurationWiring;
pub use self::launchpad::wiring::preflight::PreflightWiring;
pub use self::launchpad::wiring::runtime::RuntimeWiring;
pub use self::launchpad::Launchpad;

/// Implements the [`App`] facade.
mod app;
pub use self::app::App;

/// Re-exports the `#[strut::main]` attribute macro.
pub use strut_factory::main;
