use crate::AppConfig;
use tokio::runtime::Runtime;

/// Defines the **preflight wiring** stage of a Strut application.
///
/// This represents the final step in the startup sequence, executed immediately
/// before the application's main asynchronous logic begins. It has access to
/// both the finalized [`AppConfig`] and the configured Tokio [`Runtime`].
///
/// This stage is ideal for performing final checks or logging startup
/// announcements.
///
/// ## Customization example
///
/// You can replace the default wiring to customize or suppress the standard
/// startup announcement.
///
/// ```
/// use strut::{App, AppConfig, PreflightWiring};
/// use tokio::runtime::Runtime;
///
/// fn main() {
///     App::launchpad(async_main())
///         .with_preflight_wiring(CustomPreflightWiring)
///         .boot();
/// }
///
/// async fn async_main() {
///     println!("Executing the main logic");
/// }
///
/// struct CustomPreflightWiring;
///
/// impl PreflightWiring for CustomPreflightWiring {
///     fn announce_startup(&self, _config: &'static AppConfig, _runtime: &Runtime) {
///         // Don’t announce startup
///     }
/// }
/// ```
pub trait PreflightWiring {
    /// Runs the preflight wiring stage.
    ///
    /// This is the entry point for the stage and is not typically necessary to
    /// override directly.
    fn run(&self, config: &'static AppConfig, runtime: &Runtime) {
        // Announce startup
        self.announce_startup(config, runtime);
    }

    /// Announces that the application has started successfully.
    ///
    /// The default implementation logs a startup message using `tracing` that
    /// includes the application's name, active profile, and replica information.
    /// This log is only emitted if the `tracing` feature is enabled.
    fn announce_startup(&self, _config: &'static AppConfig, _runtime: &Runtime) {
        #[cfg(feature = "tracing")]
        {
            let replica_description: &str =
                if let Some(replica_index) = strut_core::AppReplica::index() {
                    &format!("replica #{}", replica_index)
                } else {
                    "default replica"
                };

            tracing::info!(
                "Starting {} with profile '{}' ({}, lifetime ID '{}')",
                _config.name(),
                strut_core::AppProfile::active(),
                replica_description,
                strut_core::AppReplica::lifetime_id(),
            );
        }
    }
}

/// The default `PreflightWiring` implementation used by Strut.
///
/// This struct simply uses the default behavior provided by the
/// `PreflightWiring` trait methods.
pub(crate) struct DefaultPreflightWiring;

impl PreflightWiring for DefaultPreflightWiring {}
