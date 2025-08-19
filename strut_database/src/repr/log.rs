use humantime::parse_duration;
use log::LevelFilter;
use serde::de::{Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use sqlx_core::connection::LogSettings;
use std::fmt::Formatter;
use std::time::Duration;
use strut_factory::{Deserialize as StrutDeserialize, impl_deserialize_field};

/// Closely replicates the `sqlx` crate’s [`LogSettings`] struct, providing
/// the [deserialization](serde::de::Deserialize) capability.
#[derive(Debug)]
pub(crate) struct ProxyLogSettings {
    pub(crate) statements_level: LevelFilter,
    pub(crate) slow_statements_level: LevelFilter,
    pub(crate) slow_statements_duration: Duration,
}

/// Closely replicates the `log` crate’s [`LevelFilter`] enum, providing
/// the more human-friendly [deserialization](Deserialize).
#[derive(Debug, StrutDeserialize)]
pub(crate) enum ProxyLevelFilter {
    #[strut(alias = "no")]
    Off,
    #[strut(alias = "err")]
    Error,
    #[strut(alias = "warning")]
    Warn,
    Info,
    Debug,
    Trace,
}

impl ProxyLevelFilter {
    pub(crate) fn to_log_level_filter(&self) -> LevelFilter {
        match self {
            Self::Off => LevelFilter::Off,
            Self::Error => LevelFilter::Error,
            Self::Warn => LevelFilter::Warn,
            Self::Info => LevelFilter::Info,
            Self::Debug => LevelFilter::Debug,
            Self::Trace => LevelFilter::Trace,
        }
    }
}

const _: () = {
    impl From<ProxyLogSettings> for LogSettings {
        fn from(value: ProxyLogSettings) -> Self {
            // Since the target type is non-exhaustive, we take the long road
            let mut settings = LogSettings::default();

            // Set the values we know
            settings.statements_level = value.statements_level;
            settings.slow_statements_level = value.slow_statements_level;
            settings.slow_statements_duration = value.slow_statements_duration;

            settings
        }
    }
};

const _: () = {
    impl From<ProxyLevelFilter> for LevelFilter {
        fn from(value: ProxyLevelFilter) -> Self {
            value.to_log_level_filter()
        }
    }

    impl From<&ProxyLevelFilter> for LevelFilter {
        fn from(value: &ProxyLevelFilter) -> Self {
            value.to_log_level_filter()
        }
    }
};

const _: () = {
    impl<'de> Deserialize<'de> for ProxyLogSettings {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(ProxyLogSettingsVisitor)
        }
    }

    struct ProxyLogSettingsVisitor;

    impl<'de> Visitor<'de> for ProxyLogSettingsVisitor {
        type Value = ProxyLogSettings;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of statement logging choices")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut statements_level: Option<ProxyLevelFilter> = None;
            let mut slow_statements_level: Option<ProxyLevelFilter> = None;
            let mut slow_statements_duration = None;

            while let Some(key) = map.next_key()? {
                match key {
                    ProxyLogSettingsField::statements_level => {
                        key.poll(&mut map, &mut statements_level)?
                    }
                    ProxyLogSettingsField::slow_statements_level => {
                        key.poll(&mut map, &mut slow_statements_level)?
                    }
                    ProxyLogSettingsField::slow_statements_duration => {
                        let duration_string = map.next_value::<String>()?;
                        let duration = parse_duration(&duration_string).map_err(Error::custom)?;
                        slow_statements_duration = Some(duration);
                        IgnoredAny
                    }
                    ProxyLogSettingsField::__ignore => map.next_value()?,
                };
            }

            let default = LogSettings::default();

            Ok(ProxyLogSettings {
                statements_level: statements_level
                    .map(LevelFilter::from)
                    .unwrap_or_else(|| default.statements_level),
                slow_statements_level: slow_statements_level
                    .map(LevelFilter::from)
                    .unwrap_or_else(|| default.slow_statements_level),
                slow_statements_duration: slow_statements_duration
                    .unwrap_or_else(|| default.slow_statements_duration),
            })
        }
    }

    impl_deserialize_field!(
        ProxyLogSettingsField,
        strut_deserialize::Slug::eq_as_slugs,
        statements_level,
        slow_statements_level,
        slow_statements_duration,
    );
};
