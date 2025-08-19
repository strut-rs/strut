use serde::Deserialize;
use std::time::{Duration, Instant};
use strut::{AppConfig, AppContext, AppSpindown};

#[strut::main]
async fn main() {
    // Schedule startup and spindown logic
    tokio::spawn(heartbeat_startup());
    tokio::spawn(heartbeat_spindown());

    // Simulate 10 seconds of work
    tokio::time::sleep(Duration::from_secs(10)).await;
}

/// Configuration struct that implements [`Default`] and [`Deserialize`].
#[derive(Debug, Deserialize)]
#[serde(default)]
struct HeartbeatConfig {
    period_secs: u64,
}

/// Implementing by hand to have non-trivial defaults.
impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self { period_secs: 3 }
    }
}

/// Implements main logic of the `heartbeat` component. In this case, it emits
/// an incrementing tick number at regular intervals.
async fn heartbeat_startup() {
    // Create counter
    let mut counter = 0;

    // Retrieve config
    let heartbeat_config: HeartbeatConfig = AppConfig::section("heartbeat");
    let period = Duration::from_secs(heartbeat_config.period_secs);

    // Write heartbeat events repeatedly
    while AppContext::is_alive() {
        tokio::time::sleep(period).await;
        tracing::info!("Tick {}", counter);
        counter += 1;
    }
}

/// Implements pre-shutdown logic of the `heartbeat` component. In this case, it
/// reports the total application uptime.
async fn heartbeat_spindown() {
    // Mark startup time
    let startup_time = Instant::now();

    // Register for spindown (dropping the token is sufficient to signal completion)
    let _token = AppSpindown::register("heartbeat");

    // Wait for global application context to terminate
    AppContext::terminated().await;

    // Report total uptime right before shutdown
    tracing::info!("Total uptime: {} seconds", startup_time.elapsed().as_secs());
}
