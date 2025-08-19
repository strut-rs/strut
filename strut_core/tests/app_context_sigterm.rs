mod common;

#[cfg(all(test, unix))]
mod tests {
    use crate::common::context::AppContextTestVehicle;
    use strut_core::AppContext;

    #[tokio::test]
    async fn sigterm() {
        // Given
        let mut vehicle = AppContextTestVehicle::new();

        // When
        vehicle.spawn_workload().await;
        vehicle.spawn_workload().await;

        // Then
        vehicle.assert_workloads_not_finished();

        // When
        AppContext::auto_terminate().await;
        unsafe {
            libc::raise(libc::SIGTERM);
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;

        // Then
        vehicle.assert_workloads_finished();
    }
}
