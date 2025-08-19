use sqlx::Error as SqlxError;
use sqlx_core::database::Database;
use sqlx_core::migrate::{Migrate, MigrateError, Migrator};
use sqlx_core::pool::Pool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use strut_core::{AppContext, AppSpindown, AppSpindownToken};
use strut_sync::{Gate, Latch};
use strut_util::Backoff;
use tokio::select;
use tracing::{error, info, warn};

/// An asynchronous worker that applies database migrations in the background
/// and signals completion to whoever cares to listen via the returned [`Gate`].
pub struct MigrationsWorker<DB>
where
    DB: Database,
    <DB as Database>::Connection: Migrate,
{
    name: Arc<str>,
    migrator: &'static Migrator,
    pool: Pool<DB>,
    backoff: Backoff,
    latch: Latch,
    _spindown_token: AppSpindownToken,
}

impl<DB> MigrationsWorker<DB>
where
    DB: Database,
    <DB as Database>::Connection: Migrate,
{
    /// Starts applying migrations using the given static reference to a
    /// [`migrator`](Migrator) and the given [`pool`](Pool) of database
    /// connections. The given `name` is attached to this [`MigrationsWorker`]
    /// for logging/alerting purposes.
    ///
    /// The returned [`Gate`] can be used to await for the migrations to be
    /// successfully applied.
    pub async fn start(name: impl AsRef<str>, migrator: &'static Migrator, pool: Pool<DB>) -> Gate {
        let name = Self::compose_name(name);
        let backoff = Backoff::default();
        let latch = Latch::new();
        let gate = latch.gate();
        let _spindown_token = AppSpindown::register(&name);

        let worker = Self {
            name,
            migrator,
            pool,
            backoff,
            latch,
            _spindown_token,
        };

        tokio::spawn(worker.apply());

        gate
    }

    /// Composes a human-readable name for this worker.
    fn compose_name(name: impl AsRef<str>) -> Arc<str> {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        Arc::from(format!(
            "database:migrations:{}:{}",
            name.as_ref(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ))
    }

    /// Repeatedly attempts to apply the migrations until it either succeeds or
    /// the global [`AppContext`] is terminated.
    async fn apply(self) {
        loop {
            let state = select! {
                biased;
                _ = AppContext::terminated() => ServingState::Terminated,
                result = self.migrator.run(&self.pool) => self.interpret_result(result).await,
            };

            if matches!(state, ServingState::Terminated) {
                break;
            }
        }

        // Nothing to clean up, we just have to allow the self._spindown_token to be dropped
    }

    /// Interprets the result of a single attempt to apply migrations.
    async fn interpret_result(&self, result: Result<(), MigrateError>) -> ServingState {
        match result {
            // Migrations successfully applied
            Ok(_) => {
                info!(
                    name = self.name.as_ref(),
                    "Successfully applied the database migrations",
                );

                // This is important, as it will unblock any dependent resources
                self.latch.release();

                // No further attempts are necessary
                ServingState::Terminated
            }

            // Something went wrong while applying migrations
            Err(error) => {
                // Report the situation
                self.report_error(&error);

                // Wait a bit before re-trying
                select! {
                    biased;
                    _ = AppContext::terminated() => ServingState::Terminated,
                    _ = self.backoff.sleep_next() => ServingState::Ongoing,
                }
            }
        }
    }

    /// Reports the given [`MigrateError`] depending on its perceived severity.
    fn report_error(&self, error: &MigrateError) {
        if error.is_significant() {
            error!(
                alert = true,
                name = self.name.as_ref(),
                ?error,
                error_message = %error,
                "Failed to apply the database migrations",
            );
        } else {
            warn!(
                name = self.name.as_ref(),
                ?error,
                error_message = %error,
                "Temporarily unable to apply the database migrations",
            );
        }
    }
}

/// Internal marker that indicates the state of this worker.
enum ServingState {
    Ongoing,
    Terminated,
}

/// Internal convenience trait for determining whether an `sqlx` error should be
/// perceived as significant for the logging/alerting purposes.
trait IsSignificant {
    fn is_significant(&self) -> bool;
}

impl IsSignificant for MigrateError {
    fn is_significant(&self) -> bool {
        match self {
            MigrateError::Execute(error) => error.is_significant(),
            _ => true,
        }
    }
}

impl IsSignificant for SqlxError {
    fn is_significant(&self) -> bool {
        match self {
            SqlxError::PoolTimedOut => false, // just a temporary connectivity issue
            _ => true,
        }
    }
}
