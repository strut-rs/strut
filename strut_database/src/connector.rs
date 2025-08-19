use crate::repr::handle::Handle;
use sqlx_core::database::Database;
use sqlx_core::pool::Pool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use strut_core::{AppContext, AppSpindown, AppSpindownToken};
use tracing::info;

/// Runs in the background, holds a copy of `sqlx` database connection [`Pool`]
/// created from the given [`Handle`], and closes the pooled connections once
/// the global [`AppContext`] is terminated.
///
/// In all fairness, this [`Connector`] does no “connecting” whatsoever: it
/// merely lazily initializes a [`Pool`], holds a copy of it, and returns
/// another copy to the caller. All database connectivity logic is implemented
/// by the pool itself. The main purpose of this connector is to clean up during
/// [`AppSpindown`].
pub struct Connector<DB>
where
    DB: Database,
{
    /// The globally unique name of this connector, for logging/debugging
    /// purposes.
    name: Arc<str>,
    /// The identifier of this connector’s [`Handle`], for logging/debugging
    /// purposes.
    identifier: Arc<str>,
    /// The pool of connections that this connector holds.
    pool: Pool<DB>,
    /// The canary token, which (once it goes out of scope) will inform the application
    /// that this connector gracefully completed.
    _spindown_token: AppSpindownToken,
}

impl<DB> Connector<DB>
where
    DB: Database,
{
    /// Creates a new [`Connector`] for the given [`Handle`] and sends it into
    /// background to eventually close the pooled database connections once the
    /// global [`AppContext`] is terminated.
    ///
    /// The returned [`Pool`] may be cloned and re-used as necessary.
    pub fn start<H>(handle: H) -> Pool<DB>
    where
        H: Handle<Database = DB>,
    {
        let name = Self::compose_name(&handle);
        let identifier = Arc::from(handle.identifier());
        let (connect_options, pool_options) = handle.destruct();
        let pool = pool_options.connect_lazy_with(connect_options);
        let pool_to_return = pool.clone();
        let _spindown_token = AppSpindown::register(&name);

        let connector = Self {
            name,
            identifier,
            pool,
            _spindown_token,
        };

        tokio::spawn(connector.stand_by());

        pool_to_return
    }

    /// Composes a human-readable name for this connector.
    fn compose_name(handle: &impl Handle) -> Arc<str> {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        Arc::from(format!(
            "database:connector:{}:{}",
            handle.name(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ))
    }
}

impl<DB> Connector<DB>
where
    DB: Database,
{
    /// Main, long-running function waits until the global [`AppContext`] is
    /// terminated. After that it falls into the spindown phase, where it cleans
    /// up before returning.
    async fn stand_by(self) {
        // Wait for the global context to terminate
        AppContext::terminated().await;

        // Announce spindown
        info!(
            name = self.name.as_ref(),
            identifier = self.identifier.as_ref(),
            "Closing the database connection pool",
        );

        // Close database connections
        self.pool.close().await;
    }
}
