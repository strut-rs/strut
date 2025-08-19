use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::time::Duration;
use strut_factory::impl_deserialize_field;

/// Defines a collection of fine-tune parameters for an
/// [exponential backoff](backoff::ExponentialBackoff) mechanism.
#[derive(Debug, Clone, PartialEq)]
pub struct BackoffConfig {
    pub(crate) initial_interval: Duration,
    pub(crate) max_interval: Duration,
    pub(crate) randomization_factor: f64,
    pub(crate) multiplier: f64,
    pub(crate) max_elapsed_time: Option<Duration>,
}

impl BackoffConfig {
    /// Exposes the
    /// [initial interval](backoff::ExponentialBackoffBuilder::with_initial_interval)
    /// of this exponential backoff definition.
    pub fn initial_interval(&self) -> Duration {
        self.initial_interval
    }

    /// Exposes the
    /// [max interval](backoff::ExponentialBackoffBuilder::with_max_interval)
    /// of this exponential backoff definition.
    pub fn max_interval(&self) -> Duration {
        self.max_interval
    }

    /// Exposes the
    /// [randomization factor](backoff::ExponentialBackoffBuilder::with_randomization_factor)
    /// of this exponential backoff definition.
    pub fn randomization_factor(&self) -> f64 {
        self.randomization_factor
    }

    /// Exposes the
    /// [multiplier](backoff::ExponentialBackoffBuilder::with_multiplier)
    /// of this exponential backoff definition.
    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    /// Exposes the
    /// [max elapsed time](backoff::ExponentialBackoffBuilder::with_max_elapsed_time)
    /// of this exponential backoff definition.
    pub fn max_elapsed_time(&self) -> Option<Duration> {
        self.max_elapsed_time
    }
}

impl BackoffConfig {
    fn default_initial_interval() -> Duration {
        Duration::from_secs(3)
    }

    fn default_max_interval() -> Duration {
        Duration::from_secs(60)
    }

    fn default_randomization_factor() -> f64 {
        0.5
    }

    fn default_multiplier() -> f64 {
        2.0
    }

    fn default_max_elapsed_time() -> Option<Duration> {
        None
    }
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_interval: Self::default_initial_interval(),
            max_interval: Self::default_max_interval(),
            randomization_factor: Self::default_randomization_factor(),
            multiplier: Self::default_multiplier(),
            max_elapsed_time: Self::default_max_elapsed_time(),
        }
    }
}

impl AsRef<BackoffConfig> for BackoffConfig {
    fn as_ref(&self) -> &BackoffConfig {
        self
    }
}

const _: () = {
    impl<'de> Deserialize<'de> for BackoffConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(BackoffConfigVisitor)
        }
    }

    struct BackoffConfigVisitor;

    impl<'de> Visitor<'de> for BackoffConfigVisitor {
        type Value = BackoffConfig;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of backoff configuration")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut initial_interval = None;
            let mut max_interval = None;
            let mut randomization_factor = None;
            let mut multiplier = None;
            let mut max_elapsed_time = None;

            while let Some(key) = map.next_key()? {
                match key {
                    BackoffConfigField::initial_interval => {
                        key.poll(&mut map, &mut initial_interval)?
                    }
                    BackoffConfigField::max_interval => key.poll(&mut map, &mut max_interval)?,
                    BackoffConfigField::randomization_factor => {
                        key.poll(&mut map, &mut randomization_factor)?
                    }
                    BackoffConfigField::multiplier => key.poll(&mut map, &mut multiplier)?,
                    BackoffConfigField::max_elapsed_time => {
                        key.poll(&mut map, &mut max_elapsed_time)?
                    }
                    BackoffConfigField::__ignore => map.next_value()?,
                };
            }

            Ok(BackoffConfig {
                initial_interval: initial_interval
                    .unwrap_or_else(BackoffConfig::default_initial_interval),
                max_interval: max_interval.unwrap_or_else(BackoffConfig::default_max_interval),
                randomization_factor: randomization_factor
                    .unwrap_or_else(BackoffConfig::default_randomization_factor),
                multiplier: multiplier.unwrap_or_else(BackoffConfig::default_multiplier),
                max_elapsed_time: max_elapsed_time
                    .unwrap_or_else(BackoffConfig::default_max_elapsed_time),
            })
        }
    }

    impl_deserialize_field!(
        BackoffConfigField,
        strut_deserialize::Slug::eq_as_slugs,
        initial_interval,
        max_interval,
        randomization_factor,
        multiplier,
        max_elapsed_time,
    );
};
