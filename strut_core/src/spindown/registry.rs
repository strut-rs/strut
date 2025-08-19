use crate::AppSpindownToken;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use parking_lot::Mutex;
use scopeguard::defer;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::select;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// Thread-safe growable storage for arbitrary [`SpindownWorkload`]s, with
/// ability to wait for all of them to signal back completion.
pub(crate) struct SpindownRegistry {
    registry: Mutex<Vec<SpindownWorkload>>,
    timeout: Duration,
}

impl SpindownRegistry {
    /// Internal constructor.
    pub(crate) fn new(timeout: Duration) -> Self {
        Self {
            registry: Mutex::new(Vec::new()),
            timeout,
        }
    }

    /// Adds a workload with the given name (the name needs not to be unique) to this
    /// registry and returns the corresponding [token](AppSpindownToken).
    pub(crate) fn register(&self, name: &str) -> AppSpindownToken {
        // Make a workload and extract its token
        let workload = SpindownWorkload::new(name);
        let token = workload.token();

        // Unlock internal registry and push the workload into it
        let mut registry = self.registry.lock();
        registry.push(workload);

        // Return the token
        token
    }
}

impl SpindownRegistry {
    /// Waits until all previously registered workloads have signaled
    /// completion.
    ///
    /// Returns a `usize` that indicates the count of workloads that were
    /// successfully spun down. If the spindown times out, returns
    /// [`SpindownTimeout`] instead.
    pub(crate) async fn spun_down(&self) -> Result<usize, SpindownTimeout> {
        // Announce
        info!("Spindown initiated");

        // Create a notification mechanism for the spindown timeout
        let notify_in = Arc::new(Notify::new());
        let notify_out = Arc::clone(&notify_in);

        // Start the spindown timeout
        let timeout = self.timeout;
        let timer = tokio::spawn(async move {
            tokio::time::sleep(timeout).await;
            notify_in.notify_one();
        });

        // Abort the timer in the end, no matter the outcome
        defer! { timer.abort() }

        // Start counting workloads
        let mut count = 0usize;

        // Spin down repeatedly
        loop {
            // Take currently registered workloads, leaving an empty vector in their place
            let workloads = {
                let mut registry = self.registry.lock();
                std::mem::take(&mut *registry)
            };

            // Increment counter
            count += workloads.len();

            // Claim success once there are no more registered workloads
            if workloads.is_empty() {
                info!("Spindown completed");
                return Ok(count);
            } else {
                info!(
                    "Waiting for {} registered workload(s) to complete",
                    workloads.len(),
                );
            }

            // Perform a single spindown cycle
            let result = Self::spin_down_once(workloads, &notify_out).await;

            // An error can only mean timeout: return it
            match result {
                Ok(()) => continue,
                Err(error) => {
                    return Err(SpindownTimeout {
                        spun_down: count - error.timed_out,
                        timed_out: error.timed_out,
                    });
                }
            }
        }
    }

    async fn spin_down_once(
        workloads: Vec<SpindownWorkload>,
        timeout: &Notify,
    ) -> Result<(), SpindownTimeout> {
        // Start counting workloads for this cycle
        let count = workloads.len();
        let mut remaining = count;

        // Collect the futures into an easily poll-able collection
        let mut futures = workloads
            .into_iter()
            .map(SpindownWorkloadFuture::from)
            .collect::<FuturesUnordered<_>>();

        // Repeatedly poll the futures until the spindown timer runs out
        loop {
            let state = select! {
                biased;
                _ = timeout.notified() => Self::receive_timeout(&futures),
                result = futures.next() => Self::receive_future(result, &futures),
            };

            match state {
                SpindownState::Ongoing => remaining -= 1,
                SpindownState::Completed => return Ok(()),
                SpindownState::TimedOut => {
                    return Err(SpindownTimeout {
                        spun_down: count - remaining,
                        timed_out: remaining,
                    });
                }
            }
        }
    }

    fn receive_timeout(futures: &FuturesUnordered<SpindownWorkloadFuture>) -> SpindownState {
        // Time is out: report the futures that didn’t complete in time
        for future in futures {
            error!(
                workload = future.name.as_ref(),
                "Did not complete in time during spindown",
            );
        }

        // Notify about the unfortunate circumstances
        warn!("Some workloads did not complete gracefully");

        SpindownState::TimedOut
    }

    fn receive_future(
        optional_outcome: Option<Arc<str>>,
        futures: &FuturesUnordered<SpindownWorkloadFuture>,
    ) -> SpindownState {
        // Inspect completed workload
        match optional_outcome {
            Some(workload) => {
                info!(workload = workload.as_ref(), "Completed gracefully");
            }
            None => {
                error!(
                    alert = true,
                    "Polled spindown futures while they are all already completed",
                );
            }
        }

        // Check remaining workloads
        if futures.is_empty() {
            info!("All workloads completed gracefully");
            return SpindownState::Completed;
        }

        SpindownState::Ongoing
    }
}

/// Helper enum for controlling the spindown logic flow.
enum SpindownState {
    Ongoing,
    Completed,
    TimedOut,
}

/// Little marker for spindown timeout.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct SpindownTimeout {
    spun_down: usize,
    timed_out: usize,
}

impl Display for SpindownTimeout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("failed to fully spin down all workloads within the timeout: {} completed, {} timed out")
    }
}

impl Error for SpindownTimeout {}

/// Represents an arbitrary workload registered with [`SpindownRegistry`].
///
/// A workload is merely a human-readable name that shows up in log entries
/// during the application’s spindown phase and allows to track which workloads
/// do not complete gracefully in time.
struct SpindownWorkload {
    name: Arc<str>,
    token: CancellationToken,
}

impl SpindownWorkload {
    /// Creates a new workload with the given `name`.
    fn new(name: &str) -> Self {
        Self {
            name: Arc::from(name),
            token: CancellationToken::new(),
        }
    }

    /// Creates and returns [`AppSpindownToken`] associated with this workload.
    /// Any number of tokens may be created. Punching out any of them will
    /// result in this workload being considered completed.
    fn token(&self) -> AppSpindownToken {
        AppSpindownToken::new(self.token.clone())
    }
}

impl From<SpindownWorkload> for SpindownWorkloadFuture {
    /// Consumes [`SpindownWorkload`] to produce its [future](SpindownWorkloadFuture).
    fn from(workload: SpindownWorkload) -> Self {
        let token_future = Box::pin(async move { workload.token.cancelled().await });

        SpindownWorkloadFuture {
            name: workload.name,
            token_future,
        }
    }
}

/// Custom future that wraps a `token_future` and yields the given `name`
/// whenever the wrapped future completes.
struct SpindownWorkloadFuture {
    name: Arc<str>,
    token_future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Future for SpindownWorkloadFuture {
    type Output = Arc<str>;

    /// Custom [`Future`] implementation for [`SpindownWorkloadFuture`] that
    /// simply yields the given `name` when the wrapped `token_future` (whatever
    /// it is) completes.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.token_future.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => Poll::Ready(self.name.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::time::Duration;
    use tokio::time::Instant;

    /// Helper to create a registry with a custom timeout.
    fn make_registry(timeout: Duration) -> SpindownRegistry {
        SpindownRegistry {
            registry: Mutex::new(Vec::new()),
            timeout,
        }
    }

    #[tokio::test]
    async fn no_workloads() {
        // Given
        let registry = make_registry(Duration::from_secs(5));
        let start = Instant::now();

        // When
        let count = registry.spun_down().await.unwrap();
        let elapsed = start.elapsed();

        // Then
        assert_eq!(count, 0);
        assert!(
            elapsed < Duration::from_millis(50),
            "spun_down() should return immediately when no workloads are registered",
        );
    }

    #[tokio::test]
    async fn all_workloads_complete() {
        // Given
        let registry = make_registry(Duration::from_secs(5));
        let token1 = registry.register("workload1");
        let token2 = registry.register("workload2");

        // When
        token1.punch_out();
        token2.punch_out();

        let start = Instant::now();
        let count = registry.spun_down().await.unwrap();
        let elapsed = start.elapsed();

        // Then
        assert_eq!(count, 2);
        assert!(
            elapsed < Duration::from_millis(50),
            "spun_down() should complete quickly when all workloads complete",
        );
    }

    #[tokio::test]
    async fn timeout() {
        // Given
        let registry = make_registry(Duration::from_millis(100));
        let _token = registry.register("workload_timeout");

        // When
        let start = Instant::now();
        let error = registry.spun_down().await.unwrap_err();
        let elapsed = start.elapsed();

        // Then
        assert_eq!(
            error,
            SpindownTimeout {
                spun_down: 0,
                timed_out: 1
            },
        );
        assert!(
            elapsed >= Duration::from_millis(100),
            "spun_down() should wait until timeout when workload doesn't complete",
        );
    }

    #[tokio::test]
    async fn token_drop_punch_out() {
        // Given
        let registry = make_registry(Duration::from_secs(5));
        {
            let _token = registry.register("dropped_workload");
            // _token goes out of scope here, invoking its Drop impl
        }

        // When
        let start = Instant::now();
        let count = registry.spun_down().await.unwrap();
        let elapsed = start.elapsed();

        // Then
        assert_eq!(count, 1);
        assert!(
            elapsed < Duration::from_millis(50),
            "spun_down() should complete quickly when the token is dropped",
        );
    }
}
