use crate::facade::config::initial::statics::StaticInitialConfig;
use crate::AppConfigError;
use config::{ConfigBuilder, ConfigError};
use serde::de::{DeserializeOwned, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::sync::Arc;
use strut_factory::impl_deserialize_field;

#[cfg(feature = "config-async")]
use config::builder::AsyncState;
#[cfg(not(feature = "config-async"))]
use config::builder::DefaultState;

pub mod statics;

/// Represents the application’s externalized [configuration][factor-config].
///
/// This struct is the root of the configuration tree and provides access to its
/// various sections.
///
/// ## Usage
///
/// The primary way to access the configuration is via [`AppConfig::get`], which
/// returns a static reference to the fully parsed `AppConfig` instance.
///
/// ```
/// use strut::AppConfig;
///
/// #[strut::main]
/// async fn main() {
///     // Get a reference to the initial application configuration
///     let config: &'static AppConfig = AppConfig::get();
///
///     assert_eq!(config.name(), "app");
/// }
/// ```
///
/// For accessing custom configuration sections not managed by Strut, use
/// [`AppConfig::section`].
///
/// ## Initial vs. live configuration
///
/// Strut maintains two distinct configuration concepts:
///
/// - The **initial configuration** is resolved once at startup and is immutable
///   for the lifetime of the application. [`AppConfig`] is the facade for this
///   configuration.
///
/// - The [live configuration][app-live-config] can be refreshed at runtime. It
///   is managed by [`AppLiveConfig`][app-live-config] and is intended for direct
///   use in application logic where dynamic configuration is required.
///
/// ## Structure
///
/// `AppConfig` is a flat collection of sections, where each field corresponds
/// to a top-level key in your configuration sources (e.g., a `config.toml`
/// file). Strut uses these sections to configure its integrated components like
/// `tracing` or `database`.
///
/// [factor-config]: https://www.12factor.net/config
/// [app-live-config]: crate::AppLiveConfig
#[derive(Debug, Clone)]
pub struct AppConfig {
    name: Arc<str>,

    #[cfg(feature = "tracing")]
    tracing: strut_tracing::TracingConfig,

    #[cfg(feature = "sentry")]
    sentry: strut_sentry::SentryConfig,

    #[cfg(any(
        feature = "database-mysql",
        feature = "database-postgres",
        feature = "database-sqlite",
    ))]
    database: strut_database::DatabaseConfig,

    #[cfg(feature = "rabbitmq")]
    rabbitmq: strut_rabbitmq::RabbitMqConfig,
}

/// Methods that use [`AppConfig`] as a facade.
impl AppConfig {
    /// Returns a static reference to the **initial**, immutable `AppConfig`.
    ///
    /// This configuration is resolved once during application startup. For a
    /// version that can be refreshed at runtime, see [`AppLiveConfig`].
    ///
    /// # Panics
    ///
    /// Panics if called before the configuration has been initialized, which
    /// typically happens before the `#[strut::main]` function is entered.
    ///
    /// [`AppLiveConfig`]: crate::AppLiveConfig
    pub fn get() -> &'static Self {
        StaticInitialConfig::app_config()
    }

    /// Deserializes a section of the **initial** configuration by its `key`.
    ///
    /// This method provides access to custom configuration sections that are not
    /// predefined fields on the [`AppConfig`] struct. The value is deserialized
    /// from an intermediary representation of the initial config on each call.
    /// Since the initial config is immutable, the result will be consistent.
    ///
    /// The target type `T` must implement [`Default`] and [`DeserializeOwned`].
    /// If the key is not found in the configuration, `T::default()` is returned.
    ///
    /// For a version that can be refreshed at runtime, see
    /// [`AppLiveConfig::section`][section_live].
    ///
    /// # Panics
    ///
    /// Panics if the configuration section fails to deserialize for any reason
    /// other than not being found. For a less panicky alternative, see
    /// [`try_section`].
    ///
    /// [`try_section`]: AppConfig::try_section
    /// [`DeserializeOwned`]: DeserializeOwned
    /// [section_live]: crate::AppLiveConfig::section
    pub fn section<T>(key: impl AsRef<str>) -> T
    where
        T: DeserializeOwned + Default,
    {
        let key = key.as_ref();

        Self::try_section(key).unwrap_or_else(|error| {
            panic!(
                "failed to load or parse the application configuration section '{}': {}",
                key, error,
            );
        })
    }

    /// Attempts to deserialize a section of the **initial** configuration.
    ///
    /// This is the less panicky version of [`section`]. It returns an [`Err`]
    /// of [`AppConfigError`] if deserialization fails, instead of panicking.
    ///
    /// # Panics
    ///
    /// This method will still panic if called before the configuration has been
    /// initialized. See [`get`](AppConfig::get) for details.
    ///
    /// [`section`]: AppConfig::section
    pub fn try_section<T>(key: impl AsRef<str>) -> Result<T, AppConfigError>
    where
        T: DeserializeOwned + Default,
    {
        StaticInitialConfig::proxy_config()
            .get(key.as_ref())
            .or_else(|error| match error {
                ConfigError::NotFound(_) => Ok(T::default()),
                _ => Err(AppConfigError::from(error)),
            })
    }
}

#[cfg(not(feature = "config-async"))]
impl AppConfig {
    /// Seeds the **initial configuration** from a synchronous builder.
    ///
    /// This function is called internally by Strut during startup and should
    /// not be called manually.
    ///
    /// # Panics
    ///
    /// Panics if called more than once.
    pub fn seed(config_builder: ConfigBuilder<DefaultState>) {
        let proxy_config = config_builder
            .build()
            .expect("it should be possible to synchronously build the initial proxy configuration");

        StaticInitialConfig::seed(proxy_config);
    }
}

#[cfg(feature = "config-async")]
impl AppConfig {
    /// Seeds the **initial configuration** from an asynchronous builder.
    ///
    /// This function is called internally by Strut during startup and should
    /// not be called manually.
    ///
    /// # Panics
    ///
    /// Panics if called more than once.
    pub async fn seed(config_builder: ConfigBuilder<AsyncState>) {
        let proxy_config = config_builder.build().await.expect(
            "it should be possible to asynchronously build the initial proxy configuration",
        );

        StaticInitialConfig::seed(proxy_config);
    }
}

impl AppConfig {
    /// Returns the name of the application. Defaults to `"app"`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the configuration for the `tracing` (logging) component.
    #[cfg(feature = "tracing")]
    pub fn tracing(&self) -> &strut_tracing::TracingConfig {
        &self.tracing
    }

    /// Returns the configuration for the Sentry integration.
    #[cfg(feature = "sentry")]
    pub fn sentry(&self) -> &strut_sentry::SentryConfig {
        &self.sentry
    }

    /// Returns the configuration for the database integration.
    #[cfg(any(
        feature = "database-mysql",
        feature = "database-postgres",
        feature = "database-sqlite",
    ))]
    pub fn database(&self) -> &strut_database::DatabaseConfig {
        &self.database
    }

    /// Returns the configuration for the RabbitMQ integration.
    #[cfg(feature = "rabbitmq")]
    pub fn rabbitmq(&self) -> &strut_rabbitmq::RabbitMqConfig {
        &self.rabbitmq
    }
}

impl AppConfig {
    fn default_name() -> &'static str {
        "app"
    }
}

const _: () = {
    impl<'de> Deserialize<'de> for AppConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(AppConfigVisitor)
        }
    }

    struct AppConfigVisitor;

    impl<'de> Visitor<'de> for AppConfigVisitor {
        type Value = AppConfig;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of application configuration")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut name: Option<String> = None;

            #[cfg(feature = "tracing")]
            let mut tracing = None;

            #[cfg(feature = "sentry")]
            let mut sentry = None;

            #[cfg(any(
                feature = "database-mysql",
                feature = "database-postgres",
                feature = "database-sqlite",
            ))]
            let mut database = None;

            #[cfg(feature = "rabbitmq")]
            let mut rabbitmq = None;

            while let Some(key) = map.next_key()? {
                match key {
                    AppConfigField::name => key.poll(&mut map, &mut name)?,

                    #[cfg(feature = "tracing")]
                    AppConfigField::tracing => key.poll(&mut map, &mut tracing)?,
                    #[cfg(not(feature = "tracing"))]
                    AppConfigField::tracing => map.next_value()?,

                    #[cfg(feature = "sentry")]
                    AppConfigField::sentry => key.poll(&mut map, &mut sentry)?,
                    #[cfg(not(feature = "sentry"))]
                    AppConfigField::sentry => map.next_value()?,

                    #[cfg(any(
                        feature = "database-mysql",
                        feature = "database-postgres",
                        feature = "database-sqlite",
                    ))]
                    AppConfigField::database => key.poll(&mut map, &mut database)?,
                    #[cfg(not(any(
                        feature = "database-mysql",
                        feature = "database-postgres",
                        feature = "database-sqlite",
                    )))]
                    AppConfigField::database => map.next_value()?,

                    #[cfg(feature = "rabbitmq")]
                    AppConfigField::rabbitmq => key.poll(&mut map, &mut rabbitmq)?,
                    #[cfg(not(feature = "rabbitmq"))]
                    AppConfigField::rabbitmq => map.next_value()?,

                    AppConfigField::__ignore => map.next_value()?,
                };
            }

            let name = Arc::from(name.as_deref().unwrap_or_else(|| AppConfig::default_name()));

            Ok(AppConfig {
                name,

                #[cfg(feature = "tracing")]
                tracing: tracing.unwrap_or_default(),

                #[cfg(feature = "sentry")]
                sentry: sentry.unwrap_or_default(),

                #[cfg(any(
                    feature = "database-mysql",
                    feature = "database-postgres",
                    feature = "database-sqlite",
                ))]
                database: database.unwrap_or_default(),

                #[cfg(feature = "rabbitmq")]
                rabbitmq: rabbitmq.unwrap_or_default(),
            })
        }
    }

    impl_deserialize_field!(
        AppConfigField,
        strut_deserialize::Slug::eq_as_slugs,
        name,
        tracing,
        sentry,
        rabbitmq,
        database
    );
};
