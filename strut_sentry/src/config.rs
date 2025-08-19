use humantime::parse_duration;
use secure_string::SecureString;
use serde::de::{Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::time::Duration;
use strut_factory::impl_deserialize_field;

/// Represents the application-level configuration section that covers everything
/// related to Sentry integration.
///
/// This config comes with a custom [`Deserialize`] implementation, to support more
/// human-oriented textual configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct SentryConfig {
    dsn: SecureString,
    debug: bool,
    sample_rate: f32,
    traces_sample_rate: f32,
    max_breadcrumbs: usize,
    attach_stacktrace: bool,
    shutdown_timeout: Duration,
}

impl SentryConfig {
    /// Returns the Sentry DSN (Data Source Name), which acts like a connection
    /// string. This value tells the app where to send error reports.
    pub fn dsn(&self) -> &SecureString {
        &self.dsn
    }

    /// Indicates whether Sentry debug mode is enabled.
    ///
    /// When `true`, the Sentry client will log internal operations (e.g., failed
    /// event deliveries). Useful during development or troubleshooting.
    pub fn debug(&self) -> bool {
        self.debug
    }

    /// Returns the sample rate for error event reporting (0.0 to 1.0).
    ///
    /// For example, a value of 1.0 means all errors will be reported; 0.5 means
    /// only half (randomly selected). Helps control how much data is sent.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Returns the traces sample rate (0.0 to 1.0), which controls performance
    /// tracing.
    ///
    /// This affects how often spans and transaction traces are sent to Sentry.
    /// Higher values give more observability but can increase overhead.
    pub fn traces_sample_rate(&self) -> f32 {
        self.traces_sample_rate
    }

    /// Returns the maximum number of breadcrumbs (context logs) stored per event.
    ///
    /// Breadcrumbs are small logs (like “user clicked button”) that help
    /// reconstruct what happened before an error. This setting limits how many
    /// of those are retained.
    pub fn max_breadcrumbs(&self) -> usize {
        self.max_breadcrumbs
    }

    /// Indicates whether stack traces should be automatically attached to events.
    ///
    /// When `true`, errors and certain logs will include call stacks to help
    /// identify where they originated. This improves debugging but adds overhead.
    pub fn attach_stacktrace(&self) -> bool {
        self.attach_stacktrace
    }

    /// Returns the maximum time allowed to send any remaining events before
    /// shutdown.
    ///
    /// On application exit, Sentry will attempt to flush queued events.
    /// This timeout defines how long it should wait before giving up.
    pub fn shutdown_timeout(&self) -> Duration {
        self.shutdown_timeout
    }
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            dsn: Self::default_dsn(),
            debug: Self::default_debug(),
            sample_rate: Self::default_sample_rate(),
            traces_sample_rate: Self::default_traces_sample_rate(),
            max_breadcrumbs: Self::default_max_breadcrumbs(),
            attach_stacktrace: Self::default_attach_stacktrace(),
            shutdown_timeout: Self::default_shutdown_timeout(),
        }
    }
}

impl SentryConfig {
    fn default_dsn() -> SecureString {
        "".into()
    }

    fn default_debug() -> bool {
        false
    }

    fn default_sample_rate() -> f32 {
        1.0
    }

    fn default_traces_sample_rate() -> f32 {
        0.0
    }

    fn default_max_breadcrumbs() -> usize {
        64
    }

    fn default_attach_stacktrace() -> bool {
        false
    }

    fn default_shutdown_timeout() -> Duration {
        Duration::from_secs(2)
    }
}

impl AsRef<SentryConfig> for SentryConfig {
    fn as_ref(&self) -> &SentryConfig {
        self
    }
}

const _: () = {
    impl<'de> Deserialize<'de> for SentryConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(SentryConfigVisitor)
        }
    }

    struct SentryConfigVisitor;

    impl<'de> Visitor<'de> for SentryConfigVisitor {
        type Value = SentryConfig;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of Sentry integration configuration or a string Sentry DSN")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(SentryConfig {
                dsn: SecureString::from(value),
                ..SentryConfig::default()
            })
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(SentryConfig {
                dsn: SecureString::from(value),
                ..SentryConfig::default()
            })
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut dsn = None;
            let mut debug = None;
            let mut sample_rate = None;
            let mut traces_sample_rate = None;
            let mut max_breadcrumbs = None;
            let mut attach_stacktrace = None;
            let mut shutdown_timeout = None;

            while let Some(key) = map.next_key()? {
                match key {
                    SentryConfigField::dsn => key.poll(&mut map, &mut dsn)?,
                    SentryConfigField::debug => key.poll(&mut map, &mut debug)?,
                    SentryConfigField::sample_rate => key.poll(&mut map, &mut sample_rate)?,
                    SentryConfigField::traces_sample_rate => {
                        key.poll(&mut map, &mut traces_sample_rate)?
                    }
                    SentryConfigField::max_breadcrumbs => {
                        key.poll(&mut map, &mut max_breadcrumbs)?
                    }
                    SentryConfigField::attach_stacktrace => {
                        key.poll(&mut map, &mut attach_stacktrace)?
                    }
                    SentryConfigField::shutdown_timeout => {
                        let duration_string = map.next_value::<String>()?;
                        let duration = parse_duration(&duration_string).map_err(Error::custom)?;
                        shutdown_timeout = Some(duration);
                        IgnoredAny
                    }
                    SentryConfigField::__ignore => map.next_value()?,
                };
            }

            Ok(SentryConfig {
                dsn: dsn.unwrap_or_else(SentryConfig::default_dsn),
                debug: debug.unwrap_or_else(SentryConfig::default_debug),
                sample_rate: sample_rate.unwrap_or_else(SentryConfig::default_sample_rate),
                traces_sample_rate: traces_sample_rate
                    .unwrap_or_else(SentryConfig::default_traces_sample_rate),
                max_breadcrumbs: max_breadcrumbs
                    .unwrap_or_else(SentryConfig::default_max_breadcrumbs),
                attach_stacktrace: attach_stacktrace
                    .unwrap_or_else(SentryConfig::default_attach_stacktrace),
                shutdown_timeout: shutdown_timeout
                    .unwrap_or_else(SentryConfig::default_shutdown_timeout),
            })
        }
    }

    impl_deserialize_field!(
        SentryConfigField,
        strut_deserialize::Slug::eq_as_slugs,
        dsn,
        debug,
        sample_rate,
        traces_sample_rate,
        max_breadcrumbs,
        attach_stacktrace,
        shutdown_timeout,
    );
};

#[cfg(test)]
mod tests {
    use crate::SentryConfig;
    use pretty_assertions::assert_eq;
    use secure_string::SecureString;
    use std::time::Duration;

    #[test]
    fn from_empty() {
        // Given
        let input = "{}";
        let expected_output = SentryConfig::default();

        // When
        let actual_output = serde_yml::from_str::<SentryConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_string() {
        // Given
        let input = "some_dsn";
        let expected_output = SentryConfig {
            dsn: SecureString::from("some_dsn"),
            ..SentryConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<SentryConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_sparse() {
        // Given
        let input = r#"
dsn: some_dsn
"#;
        let expected_output = SentryConfig {
            dsn: SecureString::from("some_dsn"),
            ..SentryConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<SentryConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_full() {
        // Given
        let input = r#"
dsn: some_dsn
debug: true
sample_rate: 0.5
traces_sample_rate: 0.4
max_breadcrumbs: 50
attach_stacktrace: true
shutdown_timeout: 1s 500ms
"#;
        let expected_output = SentryConfig {
            dsn: SecureString::from("some_dsn"),
            debug: true,
            sample_rate: 0.5,
            traces_sample_rate: 0.4,
            max_breadcrumbs: 50,
            attach_stacktrace: true,
            shutdown_timeout: Duration::from_millis(1500),
        };

        // When
        let actual_output = serde_yml::from_str::<SentryConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }
}
