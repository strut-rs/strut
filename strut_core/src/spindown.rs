use self::registry::SpindownRegistry;
use crate::AppSpindownToken;
use parking_lot::Mutex;
use std::sync::OnceLock;
use std::time::Duration;

mod registry;
pub mod token;

// Global singleton spindown registry
static GLOBAL: OnceLock<SpindownRegistry> = OnceLock::new();

// Spindown timeout (stored statically to allow customizing)
const DEFAULT_TIMEOUT_SECS: u64 = 2;
static TIMEOUT_SECS: Mutex<u64> = Mutex::new(DEFAULT_TIMEOUT_SECS);

/// A facade for interacting with the application’s global spindown registry.
///
/// Allows [registering](AppSpindown::register) arbitrary workloads and later
/// [waiting](AppSpindown::completed) for all registered workloads to signal
/// their graceful completion (within a configurable timeout).
///
/// ## Problem statement
///
/// Every application may choose to run asynchronous background tasks that hold
/// some kind of resource. An example here would be a pool of database connections
/// that is owned by a background task that lends access to the pool to any task
/// that requests it. However, when the application is shut down, all background
/// tasks are unceremoniously killed, which prevents proper clean-up, such as
/// closing the database connections.
///
/// ## Spindown
///
/// To solve the problem, this crate enables the following flow.
///
/// ### Main thread
///
/// The main thread:
///
/// - spawns background tasks,
/// - waits until the global [context](AppContext) is
///   [terminated](AppContext::terminated)(e.g., in a [`select`](tokio::select)
///   block, while also waiting for the main logic to finish),
/// - [waits](AppSpindown::completed) for all background tasks to signal
///   completion,
/// - returns from the `main` function.
///
/// ### Background tasks
///
/// Meanwhile, each spawned background task:
///
/// - [registers](AppSpindown::register) with the global spindown registry,
/// - also waits until the global [context](AppContext) is
///   [terminated](AppContext::terminated) (e.g., in a [`select`](tokio::select)
///   block, while also doing other work),
/// - performs clean-up procedures (e.g., closes connections, etc.),
/// - [punches out](AppSpindownToken::punch_out) the spindown token to signal
///   completion.
pub struct AppSpindown;

impl AppSpindown {
    /// Informs the application’s global spindown registry that a workload with
    /// the given name (an arbitrary human-readable string) will need to be
    /// awaited during the application spindown, giving it some time to perform
    /// clean-up.
    ///
    /// The returned [`AppSpindownToken`] must be used by the registering
    /// workload to signal back to the registry once it has gracefully completed.
    pub fn register(name: impl AsRef<str>) -> AppSpindownToken {
        // Retrieve global registry
        let registry = Self::global_registry();

        // Register workload
        registry.register(name.as_ref())
    }

    /// Allows customizing the spindown timeout for the
    /// [global singleton registry](Self::global_registry). Importantly, this
    /// method must be called early on, before any interaction with the global
    /// spindown registry, such as [registering](Self::register) a workload. If
    /// called later, this method will have no effect.
    pub fn set_timeout_secs(timeout_secs: impl Into<u64>) {
        *TIMEOUT_SECS.lock() = timeout_secs.into();
    }

    /// Collects all previously [registered](AppSpindown::register) workloads,
    /// and then waits (within a [timeout](Self::set_timeout_secs)) for them to
    /// signal completion.
    ///
    /// This function is destructive, as it consumes the internally stored list
    /// of workloads.
    ///
    /// The spindown is performed in repeated cycles (within a single shared
    /// timeout). If new workloads are registered while previous ones are being
    /// spun down, a new cycle is initiated to wait for the next batch. This is
    /// repeated until no more registered workloads are found.
    ///
    /// Importantly, this function does **not** signal to the workloads to begin
    /// their spindown. This is the job of the global
    /// [`AppContext`](crate::AppContext).
    pub async fn completed() {
        // Retrieve global registry
        let registry = Self::global_registry();

        // Repeatedly await all workloads
        let _ = registry.spun_down().await;
    }

    /// Retrieves the global (singleton) [`SpindownRegistry`], lazily
    /// initialized.
    fn global_registry() -> &'static SpindownRegistry {
        GLOBAL.get_or_init(|| {
            let timeout = Self::deduce_timeout();

            SpindownRegistry::new(timeout)
        })
    }

    /// Infers the timeout duration.
    fn deduce_timeout() -> Duration {
        // Grab lock
        let timeout_secs = *TIMEOUT_SECS.lock();

        Duration::from_secs(timeout_secs)
    }
}
