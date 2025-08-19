use crate::launchpad::wiring::configuration::DefaultConfigurationWiring;
use crate::launchpad::wiring::preflight::DefaultPreflightWiring;
use crate::launchpad::wiring::runtime::DefaultRuntimeWiring;
use crate::{ConfigurationWiring, PreflightWiring, RuntimeWiring};
use strut_config::AssemblerChoices;
use strut_core::{AppContext, AppSpindown};
use tokio::select;

pub mod wiring {
    pub mod configuration;
    pub mod preflight;
    pub mod runtime;
}

/// Configures and launches a Strut application.
///
/// The `Launchpad` uses a builder pattern to customize the application's startup
/// process before running its main asynchronous logic. The startup process is
/// divided into distinct **wiring stages**, each with a specific responsibility.
///
/// ## Wiring stages
///
/// 1.  **Configuration wiring:** Gathers all external inputs (e.g., `.env`
///     files, configuration files) to assemble the initial, immutable
///     [`AppConfig`]. Accessing the un-initialized application configuration
///     during this stage will cause a panic.
///
/// 2.  **Runtime wiring:** Takes the initial [`AppConfig`] and constructs the
///     Tokio [`Runtime`]. From this point on, the initial configuration is
///     available.
///
/// 3.  **Preflight wiring:** Performs final setup tasks using the configuration
///     and the runtime before the main application logic begins.
///
/// Once all stages are complete, the `Launchpad` executes the application's main
/// future and waits for it to complete.
///
/// [`AppConfig`]: crate::AppConfig
/// [`Runtime`]: tokio::runtime::Runtime
pub struct Launchpad<Main>
where
    Main: Future<Output = ()>,
{
    /// The application’s main asynchronous logic.
    async_main: Main,

    /// Customizable choices for assembling configuration.
    configuration_choices: AssemblerChoices,

    /// The **configuration** wiring.
    configuration_wiring: Box<dyn ConfigurationWiring>,

    /// The **runtime** wiring.
    runtime_wiring: Box<dyn RuntimeWiring>,

    /// The **preflight** wiring.
    preflight_wiring: Box<dyn PreflightWiring>,
}

impl<Main> Launchpad<Main>
where
    Main: Future<Output = ()>,
{
    /// Creates a new `Launchpad` with default wiring.
    ///
    /// The `async_main` parameter is the primary asynchronous task that defines the
    /// application's lifecycle.
    pub fn new(async_main: Main) -> Self {
        Self {
            async_main,
            configuration_choices: AssemblerChoices::default(),
            configuration_wiring: Box::new(DefaultConfigurationWiring),
            runtime_wiring: Box::new(DefaultRuntimeWiring),
            preflight_wiring: Box::new(DefaultPreflightWiring),
        }
    }
}

impl<Main> Launchpad<Main>
where
    Main: Future<Output = ()>,
{
    /// Specifies a custom name for the configuration directory.
    ///
    /// Defaults to `"config"`.
    pub fn with_config_dir(self, name: impl Into<String>) -> Self {
        Self {
            configuration_choices: AssemblerChoices {
                dir_name: Some(name.into()),
                ..self.configuration_choices
            },
            ..self
        }
    }

    /// Enables or disables configuration overrides from environment variables.
    ///
    /// Defaults to `true`.
    pub fn with_env(self, enabled: bool) -> Self {
        Self {
            configuration_choices: AssemblerChoices {
                env_enabled: enabled,
                ..self.configuration_choices
            },
            ..self
        }
    }

    /// Specifies the prefix for environment variables used as overrides.
    ///
    /// For example, with the default prefix `"APP"`, an environment variable
    /// `APP_DATABASE_URL` would override the `database.url` key in your
    /// configuration files. The prefix itself is removed from the key.
    ///
    /// This setting is only effective if environment overrides are
    /// [enabled](Launchpad::with_env).
    ///
    /// Defaults to `"APP"`.
    pub fn with_env_prefix(self, prefix: impl Into<String>) -> Self {
        Self {
            configuration_choices: AssemblerChoices {
                env_prefix: Some(prefix.into()),
                ..self.configuration_choices
            },
            ..self
        }
    }

    /// Specifies the separator used in environment variable names.
    ///
    /// This setting is only effective if environment overrides are
    /// [enabled](Launchpad::with_env).
    ///
    /// Defaults to `"_"` (a single underscore).
    pub fn with_env_separator(self, separator: impl Into<String>) -> Self {
        Self {
            configuration_choices: AssemblerChoices {
                env_separator: Some(separator.into()),
                ..self.configuration_choices
            },
            ..self
        }
    }

    /// Replaces the default **configuration** wiring with a custom implementation.
    pub fn with_configuration_wiring<W>(self, configuration_wiring: W) -> Self
    where
        W: ConfigurationWiring + 'static,
    {
        let configuration_wiring = Box::new(configuration_wiring);

        Self {
            configuration_wiring,
            ..self
        }
    }

    /// Replaces the default **runtime** wiring with a custom implementation.
    pub fn with_runtime_wiring<S>(self, runtime_wiring: S) -> Self
    where
        S: RuntimeWiring + 'static,
    {
        let runtime_wiring = Box::new(runtime_wiring);

        Self {
            runtime_wiring,
            ..self
        }
    }

    /// Replaces the default **preflight** wiring with a custom implementation.
    pub fn with_preflight_wiring<W>(self, preflight_wiring: W) -> Self
    where
        W: PreflightWiring + 'static,
    {
        let preflight_wiring = Box::new(preflight_wiring);

        Self {
            preflight_wiring,
            ..self
        }
    }
}

impl<Main> Launchpad<Main>
where
    Main: Future<Output = ()>,
{
    /// Executes the wiring stages and runs the application.
    ///
    /// This method orchestrates the entire startup process:
    /// 1. Runs the configuration, runtime, and preflight wiring stages in order.
    /// 2. Blocks on the main asynchronous logic until it completes or a
    ///    termination signal is received.
    /// 3. Manages a graceful shutdown.
    pub fn boot(self) {
        // Resolve the initial application configuration
        let config = self.configuration_wiring.run(&self.configuration_choices);

        // Make the asynchronous runtime
        let runtime = self.runtime_wiring.run(config);

        // Run the preflight steps
        self.preflight_wiring.run(config, &runtime);

        // Proceed to the application’s main asynchronous logic
        runtime.block_on(self.run_async_main());
    }

    /// Wraps the main future to handle graceful shutdown.
    ///
    /// This internal function runs the user-provided `async_main` task and listens
    /// for a termination signal from the [`AppContext`] concurrently.
    ///
    /// On exit, it ensures the `AppContext` is terminated and waits for the
    /// [`AppSpindown`] process to complete before exiting.
    async fn run_async_main(self) {
        // Run the application’s main asynchronous logic, keeping an eye on the context
        select! {
            biased;
            _ = AppContext::terminated() => {},
            _ = self.async_main => {},
        }

        // Terminate the context in case it is not terminated yet
        AppContext::terminate();

        // Wait for the application spindown to complete
        AppSpindown::completed().await;
    }
}
