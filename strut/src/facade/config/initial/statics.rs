use crate::AppConfig;
use config::Config as ProxyConfig;
use std::sync::OnceLock;

/// The statically stored initial [`AppConfig`].
static INITIAL_APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// The statically stored initial [`ProxyConfig`].
static INITIAL_PROXY_CONFIG: OnceLock<ProxyConfig> = OnceLock::new();

/// An internal facade for working with the statically stored **initial**,
/// **immutable** application configuration: resolved no more than once,
/// eagerly, during the application start-up.
///
/// Initial configuration is the basis for bootstrapping the application, and
/// thus it must fail fast. Methods on this facade panic in case of errors.
pub(crate) struct StaticInitialConfig;

impl StaticInitialConfig {
    /// Returns the statically stored initial [`AppConfig`].
    pub(crate) fn app_config() -> &'static AppConfig {
        INITIAL_APP_CONFIG.get().expect(
            "the initial application configuration should not be accessed before initialization",
        )
    }

    /// Returns the statically stored initial [`ProxyConfig`].
    pub(crate) fn proxy_config() -> &'static ProxyConfig {
        INITIAL_PROXY_CONFIG
            .get()
            .expect("the initial proxy configuration should not be accessed before initialization")
    }

    /// Eagerly deserializes [`AppConfig`] from the given [`ProxyConfig`] and
    /// stores both statically.
    pub(crate) fn seed(proxy_config: ProxyConfig) {
        // Clone the given proxy config (we need two copies) and deserialize the clone into app config
        let app_config = proxy_config
            .clone()
            .try_deserialize::<AppConfig>()
            .expect("it should be possible to deserialize the initial application configuration");

        // Store the given proxy config
        INITIAL_PROXY_CONFIG
            .set(proxy_config)
            .expect("the initial proxy configuration should not be set more than once");

        // Store the deserialized app config
        INITIAL_APP_CONFIG
            .set(app_config)
            .expect("the initial application configuration should not be set more than once");
    }
}
