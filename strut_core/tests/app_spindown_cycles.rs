#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use strut_core::{AppContext, AppSpindown};

    #[tokio::test]
    async fn cycles() {
        // Given
        let marker = Arc::new(AtomicBool::new(false));

        // Given
        tokio::spawn(outer_workload(marker.clone()));
        tokio::task::yield_now().await; // to give spawned task a chance to work

        // When
        AppContext::terminate();
        AppSpindown::completed().await;

        // Then
        assert!(marker.load(Ordering::Relaxed));
    }

    async fn outer_workload(marker: Arc<AtomicBool>) {
        // Register outer workload
        let _token_a = AppSpindown::register("workload_a");

        // Wait for the context to terminate
        AppContext::terminated().await;

        // Simulate clean-up
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Schedule inner workload while in clean-up
        tokio::spawn(inner_workload(marker));
    }

    async fn inner_workload(marker: Arc<AtomicBool>) {
        // Register inner workload
        let _token_b = AppSpindown::register("workload_b");

        // Wait for the context to terminate (moot point, but why not)
        AppContext::terminated().await;

        // Simulate clean-up
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Flip inner copy of the marker
        marker.store(true, Ordering::Relaxed);
    }
}
