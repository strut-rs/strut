use crate::{AppConfig, DotEnv};
use config::ConfigBuilder;
use strut_config::{Assembler, AssemblerChoices};

#[cfg(feature = "config-async")]
use config::builder::AsyncState;
#[cfg(not(feature = "config-async"))]
use config::builder::DefaultState;

/// Defines the **configuration wiring** stage of a Strut application.
///
/// This trait is responsible for the first phase of startup: preparing the
/// environment and assembling the initial, immutable [`AppConfig`].
///
/// The default implementation provides a standard startup sequence, but can be
/// replaced or extended for advanced customization.
///
/// ## Customization example
///
/// You can replace the default wiring to inject custom logic, such as setting
/// environment variables before the configuration is loaded.
///
/// ```
/// use strut::{App, AppConfig, ConfigurationWiring};
///
/// fn main() {
///     App::launchpad(async_main())
///         .with_configuration_wiring(CustomConfigurationWiring)
///         .boot();
/// }
///
/// async fn async_main() {
///     assert_eq!(AppConfig::get().name(), "custom-name");
/// }
///
/// struct CustomConfigurationWiring;
///
/// impl ConfigurationWiring for CustomConfigurationWiring {
///     fn prepare_environment(&self) {
///         // Set an environment variable manually
///         unsafe { std::env::set_var("APP_NAME", "custom-name") }
///     }
/// }
/// ```
pub trait ConfigurationWiring {
    /// Runs the configuration wiring stage.
    ///
    /// This is the entry point for the stage. It orchestrates the setup process
    /// by calling the other methods on this trait in a specific order. It is not
    /// typically necessary to override this method directly.
    fn run(&self, choices: &AssemblerChoices) -> &'static AppConfig {
        // Prepare the environment
        self.prepare_environment();

        // Make the config builder
        let builder = self.make_config_builder(choices);

        // Set the config builder for the live application configuration
        #[cfg(feature = "config-live")]
        self.seed_live_config(builder.clone());

        // Set the config builder for the initial application configuration
        self.seed_initial_config(builder);

        // Resolve the initial config
        let config = self.get_config();

        config
    }

    /// Prepares the process environment before configuration loading.
    ///
    /// The default implementation calls [`DotEnv::tap`] to load variables from
    /// `.env` files. This is the appropriate place to customize environment
    /// setup, for instance, by setting default values.
    fn prepare_environment(&self) {
        DotEnv::tap();
    }

    /// Creates the `ConfigBuilder` used to source configuration.
    ///
    /// The default implementation uses [`Assembler`] to construct a builder based
    /// on the provided choices, sourcing from standard file locations and
    /// environment variables. The return type depends on the `config-async` feature.
    #[cfg(not(feature = "config-async"))]
    fn make_config_builder(&self, choices: &AssemblerChoices) -> ConfigBuilder<DefaultState> {
        Assembler::make_sync_builder(choices)
    }

    /// Creates the `ConfigBuilder` used to source configuration.
    ///
    /// The default implementation uses [`Assembler`] to construct a builder based
    /// on the provided choices, sourcing from standard file locations and
    /// environment variables. The return type depends on the `config-async` feature.
    #[cfg(feature = "config-async")]
    fn make_config_builder(&self, choices: &AssemblerChoices) -> ConfigBuilder<AsyncState> {
        Assembler::make_async_builder(choices)
    }

    /// Provides the `ConfigBuilder` to the live configuration facade.
    ///
    /// This method is only available when the `config-live` feature is enabled.
    /// It passes a clone of the builder to [`AppLiveConfig`], enabling runtime
    /// configuration reloads.
    ///
    /// [`AppLiveConfig`]: crate::AppLiveConfig
    #[cfg(not(feature = "config-async"))]
    #[cfg(feature = "config-live")]
    fn seed_live_config(&self, builder: ConfigBuilder<DefaultState>) {
        crate::AppLiveConfig::set_builder(builder);
    }

    /// Provides the `ConfigBuilder` to the live configuration facade.
    ///
    /// This method is only available when the `config-live` feature is enabled.
    /// It passes a clone of the builder to [`AppLiveConfig`], enabling runtime
    /// configuration reloads.
    ///
    /// [`AppLiveConfig`]: crate::AppLiveConfig
    #[cfg(feature = "config-async")]
    #[cfg(feature = "config-live")]
    fn seed_live_config(&self, builder: ConfigBuilder<AsyncState>) {
        crate::AppLiveConfig::set_builder(builder);
    }

    /// Builds and seeds the initial, immutable `AppConfig`.
    ///
    /// This method takes ownership of the builder, builds the configuration, and
    /// stores it in a static location for the lifetime of the application.
    ///
    /// When the `config-async` feature is enabled, this method must create a
    /// temporary Tokio runtime to build the configuration, as the main
    /// application runtime has not yet been created.
    #[cfg(not(feature = "config-async"))]
    fn seed_initial_config(&self, builder: ConfigBuilder<DefaultState>) {
        AppConfig::seed(builder);
    }

    /// Builds and seeds the initial, immutable `AppConfig`.
    ///
    /// This method takes ownership of the builder, builds the configuration, and
    /// stores it in a static location for the lifetime of the application.
    ///
    /// When the `config-async` feature is enabled, this method must create a
    /// temporary Tokio runtime to build the configuration, as the main
    /// application runtime has not yet been created.
    #[cfg(feature = "config-async")]
    fn seed_initial_config(&self, builder: ConfigBuilder<AsyncState>) {
        // Construct a lean current-thread runtime to be discarded immediately after
        let tmp_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("it should be possible to build a temporary tokio runtime");

        tmp_runtime.block_on(AppConfig::seed(builder));
    }

    /// Retrieves the now-initialized static `AppConfig`.
    ///
    /// This is the final step of the wiring stage, returning a static reference
    /// to the configuration that was just built and seeded.
    fn get_config(&self) -> &'static AppConfig {
        AppConfig::get()
    }
}

/// The default `ConfigurationWiring` implementation used by Strut.
///
/// This struct simply uses the default behavior provided by the
/// `ConfigurationWiring` trait methods.
pub(crate) struct DefaultConfigurationWiring;

impl ConfigurationWiring for DefaultConfigurationWiring {}
