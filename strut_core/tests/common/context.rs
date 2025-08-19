use pretty_assertions::assert_eq;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use strut_core::AppContext;

/// Helper struct for testing [`AppContext`].
pub struct AppContextTestVehicle {
    markers: Vec<Arc<AtomicBool>>,
}

impl AppContextTestVehicle {
    /// Initializes a new test vehicle.
    pub fn new() -> Self {
        Self { markers: vec![] }
    }

    /// Spawns an async workload that gets marked as done in the background
    /// **after** the context is terminated. Keeps a marker for the workload to
    /// be able to check whether it is completed or not.
    pub async fn spawn_workload(&mut self) {
        // Create a fresh marker
        let marker = Arc::new(AtomicBool::new(false));

        // Schedule flipping the marker after the context is terminated
        tokio::spawn(Self::flip_marker_after_context_terminates(marker.clone()));

        // Save the marker for later
        self.markers.push(marker);

        // Yield to the runtime to let the spawned task kick in
        tokio::task::yield_now().await;
    }

    async fn flip_marker_after_context_terminates(marker: Arc<AtomicBool>) {
        AppContext::terminated().await;

        marker.store(true, Ordering::SeqCst);
    }

    /// Asserts that all previously spawned workloads are not yet finished.
    pub fn assert_workloads_not_finished(&mut self) {
        for marker in &self.markers {
            assert_eq!(marker.load(Ordering::SeqCst), false);
        }
    }

    /// Asserts that all previously spawned workloads are now finished.
    pub fn assert_workloads_finished(&mut self) {
        for marker in &self.markers {
            assert_eq!(marker.load(Ordering::SeqCst), true);
        }
    }
}
