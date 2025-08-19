use crate::BackoffConfig;
use backoff::backoff::Backoff as InnerBackoff;
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use parking_lot::Mutex as SyncMutex;
use std::time::Duration;

/// Thin wrapper around [`ExponentialBackoff`] that provides light-weight
/// synchronization for interior mutability, convenience method, and opinionated
/// defaults.
pub struct Backoff {
    inner: SyncMutex<ExponentialBackoff>,
}

impl Backoff {
    /// Builds a new [`Backoff`] based on the given [`BackoffConfig`].
    pub fn new(config: impl AsRef<BackoffConfig>) -> Self {
        let config = config.as_ref();
        let inner = ExponentialBackoffBuilder::new()
            .with_initial_interval(config.initial_interval())
            .with_max_interval(config.max_interval())
            .with_randomization_factor(config.randomization_factor())
            .with_multiplier(config.multiplier())
            .with_max_elapsed_time(config.max_elapsed_time())
            .build();

        Self {
            inner: SyncMutex::new(inner),
        }
    }

    /// Returns a new [`Backoff`] builder.
    pub fn builder() -> BackoffBuilder {
        BackoffBuilder::new()
    }

    /// Returns the next backoff interval.
    pub fn next(&self) -> Option<Duration> {
        self.inner.lock().next_backoff()
    }

    /// Sleeps for the next backoff interval.
    pub async fn sleep_next(&self) {
        let next_duration = self.next();

        if let Some(duration) = next_duration {
            tokio::time::sleep(duration).await;
        } else {
            tokio::task::yield_now().await;
        }
    }

    /// Resets this backoff to the initial interval.
    pub fn reset(&self) {
        self.inner.lock().reset();
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new(BackoffConfig::default())
    }
}

/// Allows to build the [`Backoff`] incrementally.
pub struct BackoffBuilder {
    config: BackoffConfig,
}

impl BackoffBuilder {
    /// Returns a new [`Backoff`] builder.
    pub fn new() -> Self {
        Self {
            config: BackoffConfig::default(),
        }
    }

    /// Sets the
    /// [initial interval](ExponentialBackoffBuilder::with_initial_interval) to
    /// the given value.
    pub fn with_initial_interval(self, initial_interval: Duration) -> Self {
        Self {
            config: BackoffConfig {
                initial_interval,
                ..self.config
            },
        }
    }

    /// Sets the
    /// [max interval](ExponentialBackoffBuilder::with_max_interval) to the
    /// given value.
    pub fn with_max_interval(self, max_interval: Duration) -> Self {
        Self {
            config: BackoffConfig {
                max_interval,
                ..self.config
            },
        }
    }

    /// Sets the
    /// [randomization factor](ExponentialBackoffBuilder::with_randomization_factor)
    /// to the given value.
    pub fn with_randomization_factor(self, randomization_factor: f64) -> Self {
        Self {
            config: BackoffConfig {
                randomization_factor,
                ..self.config
            },
        }
    }

    /// Sets the
    /// [multiplier](ExponentialBackoffBuilder::with_multiplier) to the given
    /// value.
    pub fn with_multiplier(self, multiplier: f64) -> Self {
        Self {
            config: BackoffConfig {
                multiplier,
                ..self.config
            },
        }
    }

    /// Sets the
    /// [max elapsed time](ExponentialBackoffBuilder::with_max_elapsed_time) to the given
    /// value.
    pub fn with_max_elapsed_time(self, max_elapsed_time: Option<Duration>) -> Self {
        Self {
            config: BackoffConfig {
                max_elapsed_time,
                ..self.config
            },
        }
    }

    /// Builds and returns the [`Backoff`].
    pub fn build(self) -> Backoff {
        Backoff::new(self.config)
    }
}
