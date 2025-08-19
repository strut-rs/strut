use crate::facade::config::live::statics::StaticLiveConfig;
use crate::{AppConfig, AppConfigError};
use config::ConfigBuilder;
use serde::de::DeserializeOwned;

#[cfg(feature = "config-async")]
use config::builder::AsyncState;
#[cfg(not(feature = "config-async"))]
use config::builder::DefaultState;

pub mod statics;

/// Provides access to the application's **live configuration**.
///
/// This facade complements [`AppConfig`] by offering methods to interact with a
/// configuration that can be refreshed at runtime. Use this when you need to
/// dynamically reload configuration values while the application is running.
///
/// Accessing live configuration may involve more runtime overhead compared to the
/// static, initial configuration provided by [`AppConfig`].
///
/// ## Sync vs. async
///
/// Method signatures on this struct will be `async` if the `config-async`
/// feature is enabled for the `strut` crate.
pub struct AppLiveConfig;

#[cfg(not(feature = "config-async"))]
impl AppLiveConfig {
    /// Deserializes and returns the current live `AppConfig`.
    ///
    /// This method deserializes the configuration on every call from an internal,
    /// cached representation. This cache is lazily populated on the first access.
    ///
    /// To update the configuration from its sources (e.g., files), you must first
    /// call [`refresh`]. Otherwise, this method will return the same cached values.
    /// This contrasts with [`AppConfig::get`], which always returns the immutable,
    /// initial configuration.
    ///
    /// # Panics
    ///
    /// Panics on the first access if the configuration fails to load or parse.
    /// For a non-panicking alternative, see [`try_get`].
    ///
    /// [`refresh`]: AppLiveConfig::refresh
    /// [`try_get`]: AppLiveConfig::try_get
    pub fn get() -> AppConfig {
        Self::try_get().unwrap_or_else(|error| {
            panic!(
                "failed to load or parse the application’s live configuration: {}",
                error,
            );
        })
    }

    /// Attempts to deserialize and return the current live `AppConfig`.
    ///
    /// This is the non-panicking version of [`get`]. It returns an [`Err`] of
    /// [`AppConfigError`] if loading or parsing fails.
    ///
    /// [`get`]: AppLiveConfig::get
    pub fn try_get() -> Result<AppConfig, AppConfigError> {
        StaticLiveConfig::get_config_lock()
            .read()
            .clone()?
            .try_deserialize()
            .map_err(AppConfigError::from)
    }

    /// Deserializes a section of the live configuration by its `key`.
    ///
    /// This provides access to custom configuration sections. Like [`get`], it reads
    /// from a lazily-loaded internal cache that is only updated by [`refresh`].
    ///
    /// This contrasts with [`AppConfig::section`], which accesses the immutable,
    /// initial configuration.
    ///
    /// # Panics
    ///
    /// Panics on first access if the configuration fails to load, or if the
    /// specific section fails to parse. For a non-panicking alternative, see
    /// [`try_section`].
    ///
    /// [`get`]: AppLiveConfig::get
    /// [`refresh`]: AppLiveConfig::refresh
    /// [`try_section`]: AppLiveConfig::try_section
    pub fn section<T>(key: impl AsRef<str>) -> T
    where
        T: DeserializeOwned,
    {
        let key = key.as_ref();

        Self::try_section(key).unwrap_or_else(|error| {
            panic!(
                "failed to load or parse the application’s live configuration section '{}': {}",
                key, error,
            );
        })
    }

    /// Attempts to deserialize a section of the live configuration.
    ///
    /// This is the non-panicking version of [`section`]. It returns an [`Err`] of
    /// [`AppConfigError`] if loading or parsing fails.
    ///
    /// [`section`]: AppLiveConfig::section
    pub fn try_section<T>(key: impl AsRef<str>) -> Result<T, AppConfigError>
    where
        T: DeserializeOwned,
    {
        StaticLiveConfig::get_config_lock()
            .read()
            .as_ref()
            .map_err(AppConfigError::clone)?
            .get(key.as_ref())
            .map_err(AppConfigError::from)
    }

    /// Clones and returns the underlying `ConfigBuilder`.
    ///
    /// This allows for creating a customized builder based on the current one,
    /// which can then be installed using [`set_builder`].
    ///
    /// [`set_builder`]: AppLiveConfig::set_builder
    pub fn clone_builder() -> ConfigBuilder<DefaultState> {
        StaticLiveConfig::read_builder().as_ref().unwrap().clone()
    }

    /// Replaces the underlying `ConfigBuilder` for the **live configuration**.
    ///
    /// This builder defines how configuration is sourced. It is used exclusively by
    /// the `AppLiveConfig` facade every time [`refresh`] is called.
    ///
    /// The initial, static configuration managed by [`AppConfig`] is seeded
    /// separately and is not affected by this builder.
    ///
    /// ## Timing considerations
    ///
    /// This method must be called before any live configuration is accessed.
    /// Strut's [`Launchpad`] handles this automatically during the application boot
    /// process. Manual calls are only needed for advanced customization.
    ///
    /// [`refresh`]: AppLiveConfig::refresh
    /// [`Launchpad`]: crate::Launchpad
    pub fn set_builder(builder: ConfigBuilder<DefaultState>) {
        StaticLiveConfig::set_builder(builder);
    }

    /// Reloads the live configuration from its sources.
    ///
    /// This action rebuilds the configuration using the currently set builder.
    /// After this method completes, subsequent calls to [`get`] or [`section`]
    /// will return the newly refreshed values.
    ///
    /// [`get`]: AppLiveConfig::get
    /// [`section`]: AppLiveConfig::section
    pub fn refresh() {
        StaticLiveConfig::refresh_config();
    }
}

#[cfg(feature = "config-async")]
impl AppLiveConfig {
    /// Deserializes and returns the current live `AppConfig`.
    ///
    /// This method deserializes the configuration on every call from an internal,
    /// cached representation. This cache is lazily populated on the first access.
    ///
    /// To update the configuration from its sources (e.g., files), you must first
    /// call [`refresh`]. Otherwise, this method will return the same cached values.
    /// This contrasts with [`AppConfig::get`], which always returns the immutable,
    /// initial configuration.
    ///
    /// # Panics
    ///
    /// Panics on the first access if the configuration fails to load or parse.
    /// For a non-panicking alternative, see [`try_get`].
    ///
    /// [`refresh`]: AppLiveConfig::refresh
    /// [`try_get`]: AppLiveConfig::try_get
    pub async fn get() -> AppConfig {
        Self::try_get().await.unwrap_or_else(|error| {
            panic!(
                "failed to load or parse the application’s live configuration: {}",
                error,
            );
        })
    }

    /// Attempts to deserialize and return the current live `AppConfig`.
    ///
    /// This is the non-panicking version of [`get`]. It returns an [`Err`] of
    /// [`AppConfigError`] if loading or parsing fails.
    ///
    /// [`get`]: AppLiveConfig::get
    pub async fn try_get() -> Result<AppConfig, AppConfigError> {
        StaticLiveConfig::get_config_lock()
            .await
            .read()
            .await
            .clone()?
            .try_deserialize()
            .map_err(AppConfigError::from)
    }

    /// Deserializes a section of the live configuration by its `key`.
    ///
    /// This provides access to custom configuration sections. Like [`get`], it reads
    /// from a lazily-loaded internal cache that is only updated by [`refresh`].
    ///
    /// This contrasts with [`AppConfig::section`], which accesses the immutable,
    /// initial configuration.
    ///
    /// # Panics
    ///
    /// Panics on first access if the configuration fails to load, or if the
    /// specific section fails to parse. For a non-panicking alternative, see
    /// [`try_section`].
    ///
    /// [`get`]: AppLiveConfig::get
    /// [`refresh`]: AppLiveConfig::refresh
    /// [`try_section`]: AppLiveConfig::try_section
    pub async fn section<T>(key: impl AsRef<str>) -> T
    where
        T: DeserializeOwned,
    {
        let key = key.as_ref();

        Self::try_section(key).await.unwrap_or_else(|error| {
            panic!(
                "failed to load or parse the application’s live configuration section '{}': {}",
                key, error,
            );
        })
    }

    /// Attempts to deserialize a section of the live configuration.
    ///
    /// This is the non-panicking version of [`section`]. It returns an [`Err`] of
    /// [`AppConfigError`] if loading or parsing fails.
    ///
    /// [`section`]: AppLiveConfig::section
    pub async fn try_section<T>(key: impl AsRef<str>) -> Result<T, AppConfigError>
    where
        T: DeserializeOwned,
    {
        StaticLiveConfig::get_config_lock()
            .await
            .read()
            .await
            .as_ref()
            .map_err(AppConfigError::clone)?
            .get(key.as_ref())
            .map_err(AppConfigError::from)
    }

    /// Clones and returns the underlying `ConfigBuilder`.
    ///
    /// This allows for creating a customized builder based on the current one,
    /// which can then be installed using [`set_builder`].
    ///
    /// [`set_builder`]: AppLiveConfig::set_builder
    pub fn clone_builder() -> ConfigBuilder<AsyncState> {
        StaticLiveConfig::read_builder().as_ref().unwrap().clone()
    }

    /// Replaces the underlying `ConfigBuilder` for the **live configuration**.
    ///
    /// This builder defines how configuration is sourced. It is used exclusively by
    /// the `AppLiveConfig` facade every time [`refresh`] is called.
    ///
    /// The initial, static configuration managed by [`AppConfig`] is seeded
    /// separately and is not affected by this builder.
    ///
    /// ## Timing considerations
    ///
    /// This method must be called before any live configuration is accessed.
    /// Strut's [`Launchpad`] handles this automatically during the application boot
    /// process. Manual calls are only needed for advanced customization.
    ///
    /// [`refresh`]: AppLiveConfig::refresh
    /// [`Launchpad`]: crate::Launchpad
    pub fn set_builder(builder: ConfigBuilder<AsyncState>) {
        StaticLiveConfig::set_builder(builder);
    }

    /// Reloads the live configuration from its sources.
    ///
    /// This action rebuilds the configuration using the currently set builder.
    /// After this method completes, subsequent calls to [`get`] or [`section`]
    /// will return the newly refreshed values.
    ///
    /// [`get`]: AppLiveConfig::get
    /// [`section`]: AppLiveConfig::section
    pub async fn refresh() {
        StaticLiveConfig::refresh_config().await;
    }
}
