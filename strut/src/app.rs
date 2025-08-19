use crate::Launchpad;

/// The primary entry point for launching a Strut application.
///
/// This struct provides a high-level facade for running an application.
/// Use [`App::boot`] for a quick start with default settings, or
/// [`App::launchpad`] to customize the startup process.
pub struct App;

impl App {
    /// Starts a Strut application with default settings.
    ///
    /// This function provides the simplest way to run an application. It creates a
    /// default [`Launchpad`], runs the startup process, executes the provided
    /// `async_main` future, and handles graceful shutdown.
    ///
    /// For customization options, see [`App::launchpad`].
    ///
    /// ## Example
    ///
    /// ```
    /// use strut::App;
    ///
    /// fn main() {
    ///     App::boot(async_main());
    /// }
    ///
    /// async fn async_main() {
    ///     println!("Executing the main logic");
    /// }
    /// ```
    pub fn boot<Main>(async_main: Main)
    where
        Main: Future<Output = ()>,
    {
        Self::launchpad(async_main).boot()
    }

    /// Creates a `Launchpad` for custom application startup.
    ///
    /// This is the entry point for customizing Strut's startup behavior. It
    /// returns a [`Launchpad`] instance that you can use to configure aspects like
    /// configuration sources or wiring stages before calling [`Launchpad::boot`]
    /// to run the application.
    ///
    /// ## Example
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
    ///     fn run(&self, _config: &'static AppConfig, _runtime: &Runtime) {
    ///         println!("Interjecting the preflight logic");
    ///     }
    /// }
    /// ```
    pub fn launchpad<Main>(async_main: Main) -> Launchpad<Main>
    where
        Main: Future<Output = ()>,
    {
        Launchpad::new(async_main)
    }
}
