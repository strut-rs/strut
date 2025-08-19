use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

// Global singleton token that represents the application context
static TOKEN: OnceLock<CancellationToken> = OnceLock::new();

/// Facade representing the global (singleton) application context.
///
/// The context starts in “alive” state, and can be
/// [terminated](AppContext::terminate) at any time. It’s also possible to
/// [auto-terminate](AppContext::auto_terminate) the context when an OS shutdown
/// signal is intercepted. The context can be effectively terminated only once:
/// repeated termination produces no additional effect.
///
/// Any number of asynchronous tasks may use the [`AppContext`] facade as a
/// central reference for whether the application is still alive (not in the
/// process of shutting down). A task may [wait](AppContext::terminated) for the
/// context to be terminated.
///
/// ## Example
///
/// ```rust
/// use strut_core::AppContext;
/// use tokio::task;
///
/// #[tokio::main]
/// async fn main() {
///     // Spawn a task that waits for context termination
///     let cleanup_task = task::spawn(async move {
///         // Wait...
///         AppContext::terminated().await;
///
///         // Perform some cleanup...
///     });
///
///     // Terminate manually
///     AppContext::terminate();
///
///     // Wait for cleanup to complete
///     cleanup_task.await.unwrap();
/// }
/// ```
pub struct AppContext;

impl AppContext {
    /// Internal chokepoint for accessing the global singleton [`TOKEN`].
    fn token() -> &'static CancellationToken {
        TOKEN.get_or_init(CancellationToken::new)
    }

    /// Blocks until the global application context is terminated.
    ///
    /// Any number of tasks may await on this method. If a task starts waiting
    /// on this method after the context has been terminated, the returned
    /// future completes immediately.
    pub async fn terminated() {
        Self::token().cancelled().await;
    }

    /// Terminates the global application context. If the context is already
    /// terminated, no additional effect is produced beyond a `tracing` event.
    ///
    /// When the context is terminated, all tasks
    /// [waiting](AppContext::terminated) on it will unblock.
    pub fn terminate() {
        info!("Terminating application context");

        Self::token().cancel();
    }

    /// Schedules listening for the OS shutdown signals, which
    /// [replaces](AppContext::listen_for_shutdown_signals) the default shutdown
    /// behavior of this entire OS process. After this method returns, the first
    /// intercepted OS shutdown signal will [terminate](AppContext::terminate)
    /// this context.
    ///
    /// [Take note](AppContext::listen_for_shutdown_signals) of the consequences
    /// of replacing the default process shutdown behavior.
    ///
    /// Repeated calls to this method produce no additional effect.
    ///
    /// This method must be awaited to ensure that signal listening has already
    /// started by the time the returned future completes.
    pub async fn auto_terminate() {
        // Guard against multiple calls to this method
        static CALLED: AtomicBool = AtomicBool::new(false);

        // If already called, pull a no-op
        if CALLED.swap(true, Ordering::Relaxed) {
            return;
        }

        // Schedule listening for OS shutdown signals
        tokio::spawn(Self::listen_for_shutdown_signals());

        // Yield to runtime to ensure the task spawned above has time to start working
        tokio::task::yield_now().await;
    }

    /// Reports whether the global application context has been terminated as of
    /// this moment.
    ///
    /// This method is not suitable for waiting for it to be terminated. For
    /// such purposes, use [`AppContext::terminated`].
    pub fn is_terminated() -> bool {
        Self::token().is_cancelled()
    }

    /// Reports whether the global application context has **not** yet been
    /// terminated as of this moment.
    ///
    /// This method is not suitable for waiting for it to be terminated. For
    /// such purposes, use [`AppContext::terminated`].
    pub fn is_alive() -> bool {
        !Self::token().is_cancelled()
    }

    /// **Replaces** the default shutdown behavior of this entire OS process by
    /// subscribing to the OS shutdown signals. Upon receiving the _first_
    /// shutdown signal, prevents normal process termination and instead cancels
    /// the global application context.
    ///
    /// After the first shutdown signal is handled, this method starts listening
    /// for repeated signals of the same kind. When such repeated signal is
    /// intercepted, the process exits immediately with a non-zero status code.
    ///
    /// There is a minuscule delay between the initial signal is handled and
    /// listening starts for repeated signals: within that delay it is possible
    /// that a repeated signal may fly through un-handled.
    ///
    /// ## Shutdown signals
    ///
    /// This method hijacks both `SIGINT` and `SIGTERM` on Unix platforms, and
    /// the `ctrl_c` action on non-Unix platforms.
    ///
    /// ## Usage notes
    ///
    /// Calling this method is a one-way street. After it starts executing,
    /// there is no way to restore the original shutdown behavior for this
    /// process.
    ///
    /// There is no benefit (and theoretically no harm) in calling this method
    /// more than once in the same process (e.g., from multiple asynchronous
    /// tasks).
    ///
    /// This method never returns. The future it generates will either get
    /// aborted, or an [`exit`](std::process::exit) call will terminate the
    /// whole process.
    async fn listen_for_shutdown_signals() -> ! {
        // Wait for first shutdown signal
        Self::wait_for_shutdown_signal().await;

        // Report
        info!("Shutdown signal intercepted");

        // On first shutdown signal, cancel the global token
        Self::token().cancel();

        // Wait for any subsequent shutdown signal
        Self::wait_for_shutdown_signal().await;

        // Report
        warn!("Repeated shutdown signal intercepted; exiting");

        // Exit forcibly
        std::process::exit(1);
    }

    /// Waits for the next OS shutdown signal on a Unix platform.
    #[cfg(unix)]
    async fn wait_for_shutdown_signal() {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        let mut sigterm = signal(SignalKind::terminate()).unwrap();

        tokio::select! {
            biased; // no need to pay for randomized branch checking
            _ = sigint.recv() => {}
            _ = sigterm.recv() => {}
        }
    }

    /// Waits for the next `ctrl_c` action on a non-Unix platform.
    #[cfg(not(unix))]
    async fn wait_for_shutdown_signal() {
        tokio::signal::ctrl_c().await.unwrap();
    }
}
