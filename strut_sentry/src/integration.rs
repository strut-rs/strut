use crate::SentryConfig;
use sentry::ClientInitGuard as SentryGuard;
use strut_core::{AppContext, AppProfile, AppReplica, AppSpindown, AppSpindownToken};
use tokio::runtime::Runtime;

/// A facade for integrating with Sentry.
pub struct SentryIntegration;

impl SentryIntegration {
    /// Initializes Sentry integration and returns the
    /// [client guard](SentryGuard).
    ///
    /// The behavior of the Sentry client is partly configurable from the provided
    /// [`SentryConfig`].
    pub fn init(config: impl AsRef<SentryConfig>) -> SentryGuard {
        let config = config.as_ref();

        let guard = sentry::init((
            config.dsn().unsecure(),
            sentry::ClientOptions {
                debug: config.debug(),
                release: sentry::release_name!(),
                environment: Some(AppProfile::active().as_str().into()),
                sample_rate: config.sample_rate(),
                traces_sample_rate: config.traces_sample_rate(),
                max_breadcrumbs: config.max_breadcrumbs(),
                attach_stacktrace: config.attach_stacktrace(),
                shutdown_timeout: config.shutdown_timeout(),
                ..Default::default()
            },
        ));

        sentry::configure_scope(|scope| {
            scope.set_tag(
                "replica_index",
                AppReplica::index()
                    .map(|index| index.to_string())
                    .unwrap_or_else(|| "unset".into()),
            );

            scope.set_tag("replica_lifetime_id", AppReplica::lifetime_id());
        });

        guard
    }

    /// Schedules flushing of unsend Sentry events (if any) after the
    /// [application context](AppContext) is terminated. The flushing is
    /// triggered by dropping the given [Sentry guard](SentryGuard).
    ///
    /// This scheduling involves spawning an asynchronous task. To avoid
    /// accidentally calling this function outside of Tokio context, this
    /// function explicitly takes a [runtime](Runtime) as an argument.
    pub fn schedule_flushing(runtime: &Runtime, sentry_guard: SentryGuard) {
        let spindown_token = AppSpindown::register("sentry-integration");

        runtime.spawn(Self::await_shutdown(sentry_guard, spindown_token));
    }

    /// Awaits for [`AppContext`] to get terminated, then drops the provided
    /// Sentry guard. Dropping the guard flushes unsent Sentry events, however it
    /// does so synchronously (blocks the current thread). To let the flushing
    /// run asynchronously, we [off-load](tokio::task::spawn_blocking) it to a
    /// blocking thread pool.
    async fn await_shutdown(sentry_guard: SentryGuard, spindown_token: AppSpindownToken) {
        // Wait until the global context is terminated
        AppContext::terminated().await;

        // Initiate dropping of Sentry guard on a blocking thread pool
        tokio::task::spawn_blocking(move || Self::drop_guard(sentry_guard, spindown_token));
    }

    /// Drops the given [Sentry guard](SentryGuard) first, then the given
    /// [`AppSpindownToken`].
    fn drop_guard(guard: SentryGuard, _spindown_token: AppSpindownToken) {
        // Drop the Sentry guard: this will synchronously flush all events within Sentry’s own timeout
        drop(guard);

        // The spindown token will punch itself out when dropped
    }
}
