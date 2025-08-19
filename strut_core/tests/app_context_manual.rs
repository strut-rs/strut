mod common;

#[cfg(test)]
mod tests {
    use crate::common::context::AppContextTestVehicle;
    use strut_core::AppContext;

    #[tokio::test]
    async fn manual() {
        // Given
        let mut vehicle = AppContextTestVehicle::new();

        // When
        vehicle.spawn_workload().await;
        vehicle.spawn_workload().await;

        // Then
        vehicle.assert_workloads_not_finished();

        // When
        AppContext::terminate();
        tokio::task::yield_now().await;

        // Then
        vehicle.assert_workloads_finished();
    }
}
