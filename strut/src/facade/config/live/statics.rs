use crate::AppConfigError;
use config::{Config as ProxyConfig, ConfigBuilder};
use parking_lot::{lock_api::RwLockReadGuard, RawRwLock, RwLock as SyncRwLock};
use std::sync::OnceLock;

#[cfg(not(feature = "config-async"))]
use config::builder::DefaultState;

#[cfg(feature = "config-async")]
use {
    config::builder::AsyncState,
    tokio::sync::{OnceCell, RwLock as AsyncRwLock},
};

/// An internal facade for working with the statically stored **live**,
/// **mutable** application configuration: resolved as many times as requested,
/// lazily, throughout the application’s runtime.
///
/// Live configuration is only exposed for use by downstream code. It is not
/// used inside this crate.
///
/// Methods on this facade tend to return appropriate errors instead of
/// panicking.
pub(crate) struct StaticLiveConfig;

#[cfg(not(feature = "config-async"))]
static BUILDER: OnceLock<SyncRwLock<Option<ConfigBuilder<DefaultState>>>> = OnceLock::new();

#[cfg(not(feature = "config-async"))]
impl StaticLiveConfig {
    pub(crate) fn refresh_config() {
        *Self::get_config_lock().write() = Self::load_config();
    }

    pub(crate) fn get_config_lock() -> &'static SyncRwLock<Result<ProxyConfig, AppConfigError>> {
        static CONFIG: OnceLock<SyncRwLock<Result<ProxyConfig, AppConfigError>>> = OnceLock::new();

        CONFIG.get_or_init(|| SyncRwLock::new(Self::load_config()))
    }

    fn load_config() -> Result<ProxyConfig, AppConfigError> {
        Self::read_builder()
            .as_ref()
            .unwrap()
            .build_cloned()
            .map_err(AppConfigError::from)
    }

    pub(crate) fn read_builder()
    -> RwLockReadGuard<'static, RawRwLock, Option<ConfigBuilder<DefaultState>>> {
        // Obtain the read-level lock
        let guard = Self::get_builder_lock().read();

        // Ensure this is called with appropriate timing
        if guard.is_none() {
            panic!("the application’s configuration accessed before the builder is set");
        }

        guard
    }

    pub(crate) fn set_builder(builder: ConfigBuilder<DefaultState>) {
        // Obtain the write-level lock
        let mut guard = Self::get_builder_lock().write();

        // Set the builder
        *guard = Some(builder);
    }

    fn get_builder_lock() -> &'static SyncRwLock<Option<ConfigBuilder<DefaultState>>> {
        BUILDER.get_or_init(|| SyncRwLock::new(None))
    }
}

#[cfg(feature = "config-async")]
static BUILDER: OnceLock<SyncRwLock<Option<ConfigBuilder<AsyncState>>>> = OnceLock::new();

#[cfg(feature = "config-async")]
impl StaticLiveConfig {
    pub(crate) async fn refresh_config() {
        *Self::get_config_lock().await.write().await = Self::load_config().await;
    }

    pub(crate) async fn get_config_lock()
    -> &'static AsyncRwLock<Result<ProxyConfig, AppConfigError>> {
        static CONFIG: OnceCell<AsyncRwLock<Result<ProxyConfig, AppConfigError>>> =
            OnceCell::const_new();

        CONFIG
            .get_or_init(|| async { AsyncRwLock::new(Self::load_config().await) })
            .await
    }

    async fn load_config() -> Result<ProxyConfig, AppConfigError> {
        Self::read_builder()
            .as_ref()
            .unwrap()
            .build_cloned()
            .await
            .map_err(AppConfigError::from)
    }

    pub(crate) fn read_builder()
    -> RwLockReadGuard<'static, RawRwLock, Option<ConfigBuilder<AsyncState>>> {
        // Obtain the read-level lock
        let guard = Self::get_builder_lock().read();

        // Ensure this is called with appropriate timing
        if guard.is_none() {
            panic!("the application’s configuration accessed before the builder is set");
        }

        guard
    }

    pub(crate) fn set_builder(builder: ConfigBuilder<AsyncState>) {
        // Obtain the write-level lock
        let mut guard = Self::get_builder_lock().write();

        // Set the builder
        *guard = Some(builder);
    }

    fn get_builder_lock() -> &'static SyncRwLock<Option<ConfigBuilder<AsyncState>>> {
        BUILDER.get_or_init(|| SyncRwLock::new(None))
    }
}
