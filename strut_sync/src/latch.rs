use tokio_util::sync::CancellationToken;

/// A synchronization primitive that can be released exactly once, notifying all
/// associated [`Gate`]s. This is intended for one-shot notifications or
/// barriers in asynchronous contexts.
///
/// ## Simple example
///
/// ```
/// use strut_sync::{Gate, Latch};
///
/// # tokio_test::block_on(async {
///
/// // Make a latch
/// let latch = Latch::new();
///
/// // Derive a gate from it
/// let gate = latch.gate();
///
/// // Spawn an asynchronous task
/// tokio::spawn(async move {
///     // Perform some asynchronous work
///     println!("This will print first");
///
///     // Signal completion
///     latch.release();
/// });
///
/// // Wait for the completion signal
/// gate.opened().await;
///
/// println!("Asynchronous task completed!")
/// # })
/// ```
///
/// ## Full example
///
/// ```
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicU8, Ordering};
/// use strut_sync::{Gate, Latch};
/// use pretty_assertions::assert_eq;
///
/// # tokio_test::block_on(async {
///
/// // Make a latch
/// let latch = Latch::new();
///
/// // Derive any number of gates from it
/// let gate_a = latch.gate();
/// let gate_b = latch.gate();
/// let gate_c = gate_b.clone();
///
/// // Create a marker
/// let marker = Arc::new(AtomicU8::new(0));
///
/// // Spawn tasks that increment the marker
/// tokio::spawn(increment_marker(gate_a, marker.clone()));
/// tokio::spawn(increment_marker(gate_b, marker.clone()));
/// tokio::spawn(increment_marker(gate_c, marker.clone()));
///
/// // Give the tasks a chance to start waiting
/// tokio::task::yield_now().await;
///
/// // Nothing should have happened yet
/// assert_eq!(marker.load(Ordering::Relaxed), 0);
///
/// // Release the latch
/// latch.release();
///
/// // Give the tasks a chance to wake up
/// tokio::task::yield_now().await;
///
/// // Marker should have been increased three times by now
/// assert_eq!(marker.load(Ordering::Relaxed), 3);
///
/// # });
///
/// // Helper function
/// async fn increment_marker(gate: Gate, marker: Arc<AtomicU8>) {
///     // Wait for the gate to open
///     gate.opened().await;
///
///     // Increment the marker
///     marker.fetch_add(1, Ordering::Relaxed);
/// }
/// ```
#[derive(Debug, Default, Clone)]
pub struct Latch {
    token: CancellationToken,
}

/// A single-release barrier that is [opened](Gate::opened) when the associated
/// [`Latch`] is [released](Latch::release).
///
/// This gate can be cheaply cloned and awaited on by any number of asynchronous
/// tasks at any time.
#[derive(Debug, Clone)]
pub struct Gate {
    token: CancellationToken,
}

impl Latch {
    /// Returns a brand new, unreleased [`Latch`].
    pub fn new() -> Self {
        let token = CancellationToken::new();

        Self { token }
    }

    /// Returns a new [`Gate`] handle associated with this [`Latch`]. Multiple
    /// gates can be created and awaited independently, all linked to the same
    /// single-release latch.
    pub fn gate(&self) -> Gate {
        Gate {
            token: self.token.clone(),
        }
    }

    /// Permanently releases this [`Latch`], notifying all associated [`Gate`]s.
    /// Subsequent calls have no additional effect.
    pub fn release(&self) {
        self.token.cancel();
    }
}

impl Gate {
    /// Waits asynchronously until the associated [`Latch`] is
    /// [released](Latch::release). Resolves immediately if the latch has
    /// already been released.
    pub async fn opened(&self) {
        self.token.cancelled().await;
    }

    /// Reports whether the associated [`Latch`] has been released.
    pub fn is_open(&self) -> bool {
        self.token.is_cancelled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_work_and_complete() {
        let (latch, gate, marker) = make_objects();

        tokio::spawn(work_and_release(latch));
        tokio::spawn(await_opened_and_flip_marker(gate, marker.clone()));

        sleep_a_little().await;

        assert!(marker.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_multi_work_and_complete() {
        let (latch, gate, marker) = make_objects();

        tokio::spawn(work_and_release(latch.clone()));
        tokio::spawn(work_and_release(latch.clone()));
        tokio::spawn(await_opened_and_flip_marker(gate, marker.clone()));

        sleep_a_little().await;

        assert!(marker.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_work_and_complete_reordered() {
        let (latch, gate, marker) = make_objects();

        tokio::spawn(await_opened_and_flip_marker(gate, marker.clone()));
        tokio::spawn(work_and_release(latch));

        sleep_a_little().await;

        assert!(marker.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_multi_monitor_work_and_complete() {
        let latch = Latch::new();
        let gate_a = latch.gate();
        let gate_b = gate_a.clone();
        let marker_a = Arc::new(AtomicBool::new(false));
        let marker_b = Arc::new(AtomicBool::new(false));

        tokio::spawn(work_and_release(latch));
        tokio::spawn(await_opened_and_flip_marker(gate_a, marker_a.clone()));
        tokio::spawn(await_opened_and_flip_marker(gate_b, marker_b.clone()));

        sleep_a_little().await;

        assert!(marker_a.load(Ordering::Relaxed));
        assert!(marker_b.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_multi_completion_work_and_complete() {
        let latch = Latch::new();
        let gate = latch.gate();
        let marker_a = Arc::new(AtomicBool::new(false));
        let marker_b = Arc::new(AtomicBool::new(false));

        tokio::spawn(work_a_lot(latch.clone()));
        tokio::spawn(work_and_release(latch.clone()));
        tokio::spawn(await_opened_and_flip_marker(gate.clone(), marker_a.clone()));
        tokio::spawn(await_opened_and_flip_marker(gate.clone(), marker_b.clone()));

        sleep_a_little().await;

        assert!(marker_a.load(Ordering::Relaxed));
        assert!(marker_b.load(Ordering::Relaxed));
    }

    fn make_objects() -> (Latch, Gate, Arc<AtomicBool>) {
        let latch = Latch::new();
        let gate = latch.gate();

        (latch, gate, Arc::new(AtomicBool::new(false)))
    }

    async fn work_and_release(latch: Latch) {
        tokio::time::sleep(Duration::from_millis(2)).await;
        latch.release();
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }

    async fn work_a_lot(_completion: Latch) {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }

    async fn await_opened_and_flip_marker(gate: Gate, marker: Arc<AtomicBool>) {
        gate.opened().await;
        marker.store(true, Ordering::Relaxed);
    }

    async fn sleep_a_little() {
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
