use crate::{FormatFlavor, Verbosity};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::fmt::Formatter;
use strut_factory::impl_deserialize_field;

pub mod flavor;
pub mod verbosity;

/// Represents the application-level configuration section that covers everything
/// related to pre-configuring the [formatted layer](tracing_subscriber::fmt::Layer)
/// provided by the `tracing` crate. In essence, this is the application
/// **logging** configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracingConfig {
    verbosity: Verbosity,
    flavor: FormatFlavor,
    color: bool,
    show_timestamp: bool,
    show_target: bool,
    show_file: bool,
    show_line_number: bool,
    show_level: bool,
    show_thread_id: bool,
    show_thread_name: bool,
    #[cfg(feature = "json")]
    flatten_json: bool,
    targets: BTreeMap<String, Verbosity>,
}

impl TracingConfig {
    /// Merges an extra per-target [`Verbosity`] level into this config.
    pub fn with_target(
        mut self,
        target: impl Into<String>,
        verbosity: impl Into<Verbosity>,
    ) -> Self {
        self.targets.insert(target.into(), verbosity.into());

        self
    }

    /// Merges extra per-target [`Verbosity`] levels into this config.
    pub fn with_targets<T, L>(mut self, targets: impl IntoIterator<Item = (T, L)>) -> Self
    where
        T: Into<String>,
        L: Into<Verbosity>,
    {
        for (target, verbosity) in targets.into_iter() {
            self.targets.insert(target.into(), verbosity.into());
        }

        self
    }
}

impl TracingConfig {
    /// Reports the root [verbosity level](Verbosity) for this logging
    /// configuration.
    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    /// Reports the [formatting flavor](FormatFlavor) for this logging
    /// configuration.
    pub fn flavor(&self) -> FormatFlavor {
        self.flavor
    }

    /// Reports whether this logging configuration enables
    /// [colored](tracing_subscriber::fmt::Layer::with_ansi) output.
    pub fn color(&self) -> bool {
        self.color
    }

    /// Reports whether this logging configuration includes the
    /// [timestamp](tracing_subscriber::fmt::Layer::without_time) in the output.
    pub fn show_timestamp(&self) -> bool {
        self.show_timestamp
    }

    /// Reports whether this logging configuration includes the
    /// [target](tracing_subscriber::fmt::Layer::with_target) in the output.
    pub fn show_target(&self) -> bool {
        self.show_target
    }

    /// Reports whether this logging configuration includes the
    /// [file](tracing_subscriber::fmt::Layer::with_file) in the output.
    pub fn show_file(&self) -> bool {
        self.show_file
    }

    /// Reports whether this logging configuration includes the
    /// [line number](tracing_subscriber::fmt::Layer::with_line_number) in the
    /// output.
    pub fn show_line_number(&self) -> bool {
        self.show_line_number
    }

    /// Reports whether this logging configuration includes the
    /// [level](tracing_subscriber::fmt::Layer::with_level) in the output.
    pub fn show_level(&self) -> bool {
        self.show_level
    }

    /// Reports whether this logging configuration includes the
    /// [thread ID](tracing_subscriber::fmt::Layer::with_thread_ids) in the
    /// output.
    pub fn show_thread_id(&self) -> bool {
        self.show_thread_id
    }

    /// Reports whether this logging configuration includes the
    /// [thread name](tracing_subscriber::fmt::Layer::with_thread_names) in the
    /// output.
    pub fn show_thread_name(&self) -> bool {
        self.show_thread_name
    }

    /// Reports whether this logging configuration flattens the JSON output.
    #[cfg(feature = "json")]
    pub fn flatten_json(&self) -> bool {
        self.flatten_json
    }

    /// Reports the
    /// [customized](tracing_subscriber::filter::targets::Targets::with_targets)
    /// per-[target](tracing_subscriber::filter::targets::Targets) verbosity for
    /// this logging configuration.
    pub fn targets(&self) -> &BTreeMap<String, Verbosity> {
        &self.targets
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::default(),
            flavor: FormatFlavor::default(),
            color: Self::default_color(),
            show_timestamp: Self::default_show_timestamp(),
            show_target: Self::default_show_target(),
            show_file: Self::default_show_file(),
            show_line_number: Self::default_show_line_number(),
            show_level: Self::default_show_level(),
            show_thread_id: Self::default_show_thread_id(),
            show_thread_name: Self::default_show_thread_name(),
            #[cfg(feature = "json")]
            flatten_json: Self::default_flatten_json(),
            targets: BTreeMap::default(),
        }
    }
}

impl TracingConfig {
    fn default_color() -> bool {
        true
    }

    fn default_show_timestamp() -> bool {
        true
    }

    fn default_show_target() -> bool {
        true
    }

    fn default_show_file() -> bool {
        false
    }

    fn default_show_line_number() -> bool {
        false
    }

    fn default_show_level() -> bool {
        true
    }

    fn default_show_thread_id() -> bool {
        true
    }

    fn default_show_thread_name() -> bool {
        false
    }

    #[cfg(feature = "json")]
    fn default_flatten_json() -> bool {
        true
    }
}

impl AsRef<TracingConfig> for TracingConfig {
    fn as_ref(&self) -> &TracingConfig {
        self
    }
}

const _: () = {
    impl<'de> Deserialize<'de> for TracingConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(TracingConfigVisitor)
        }
    }

    struct TracingConfigVisitor;

    impl<'de> Visitor<'de> for TracingConfigVisitor {
        type Value = TracingConfig;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of tracing (logging) configuration")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut level = None;
            let mut flavor = None;
            let mut color = None;
            let mut show_timestamp = None;
            let mut show_target = None;
            let mut show_file = None;
            let mut show_line_number = None;
            let mut show_level = None;
            let mut show_thread_id = None;
            let mut show_thread_name = None;
            #[cfg(feature = "json")]
            let mut flatten_json = None;
            let mut targets = None;

            while let Some(key) = map.next_key()? {
                match key {
                    TracingConfigField::verbosity => key.poll(&mut map, &mut level)?,
                    TracingConfigField::flavor => key.poll(&mut map, &mut flavor)?,
                    TracingConfigField::color => key.poll(&mut map, &mut color)?,
                    TracingConfigField::show_timestamp => {
                        key.poll(&mut map, &mut show_timestamp)?
                    }
                    TracingConfigField::show_target => key.poll(&mut map, &mut show_target)?,
                    TracingConfigField::show_file => key.poll(&mut map, &mut show_file)?,
                    TracingConfigField::show_line_number => {
                        key.poll(&mut map, &mut show_line_number)?
                    }
                    TracingConfigField::show_level => key.poll(&mut map, &mut show_level)?,
                    TracingConfigField::show_thread_id => {
                        key.poll(&mut map, &mut show_thread_id)?
                    }
                    TracingConfigField::show_thread_name => {
                        key.poll(&mut map, &mut show_thread_name)?
                    }
                    #[cfg(feature = "json")]
                    TracingConfigField::flatten_json => key.poll(&mut map, &mut flatten_json)?,
                    #[cfg(not(feature = "json"))]
                    TracingConfigField::flatten_json => map.next_value()?,
                    TracingConfigField::targets => key.poll(&mut map, &mut targets)?,
                    TracingConfigField::__ignore => map.next_value()?,
                };
            }

            Ok(TracingConfig {
                verbosity: level.unwrap_or_default(),
                flavor: flavor.unwrap_or_default(),
                color: color.unwrap_or_else(TracingConfig::default_color),
                show_timestamp: show_timestamp
                    .unwrap_or_else(TracingConfig::default_show_timestamp),
                show_target: show_target.unwrap_or_else(TracingConfig::default_show_target),
                show_file: show_file.unwrap_or_else(TracingConfig::default_show_file),
                show_line_number: show_line_number
                    .unwrap_or_else(TracingConfig::default_show_line_number),
                show_level: show_level.unwrap_or_else(TracingConfig::default_show_level),
                show_thread_id: show_thread_id
                    .unwrap_or_else(TracingConfig::default_show_thread_id),
                show_thread_name: show_thread_name
                    .unwrap_or_else(TracingConfig::default_show_thread_name),
                #[cfg(feature = "json")]
                flatten_json: flatten_json.unwrap_or_else(TracingConfig::default_flatten_json),
                targets: targets.unwrap_or_default(),
            })
        }
    }

    impl_deserialize_field!(
        TracingConfigField,
        strut_deserialize::Slug::eq_as_slugs,
        verbosity | level,
        flavor | flavour,
        color
            | with_color
            | colour
            | with_colour
            | show_color
            | show_colour
            | show_colors
            | show_colours,
        show_timestamp | with_timestamp,
        show_target | with_target,
        show_file | with_file,
        show_line_number | show_line | with_line | with_line_number,
        show_level | with_level,
        show_thread_id | with_thread_id,
        show_thread_name | with_thread_name,
        flatten_json | flat_json | with_flat_json,
        targets | custom_targets | target_verbosity,
    );
};

#[cfg(test)]
mod tests {
    use crate::{FormatFlavor, TracingConfig, Verbosity};
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;

    #[test]
    fn from_empty() {
        // Given
        let input = "{}";
        let expected_output = TracingConfig::default();

        // When
        let actual_output = serde_yml::from_str::<TracingConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_sparse() {
        // Given
        let input = r#"
verbosity: off
"#;
        let expected_output = TracingConfig {
            verbosity: Verbosity::Off,
            ..TracingConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<TracingConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_full() {
        // Given
        let input = r#"
verbosity: warn
flavor: pretty
show_color: false
show_timestamp: false
show_target: false
show_file: true
show_line_number: true
show_level: false
show_thread_id: false
show_thread_name: true
flatten_json: true
targets:
    crate_a: off
    crate_b::module: error
"#;
        let expected_output = TracingConfig {
            verbosity: Verbosity::Warn,
            flavor: FormatFlavor::Pretty,
            color: false,
            show_timestamp: false,
            show_target: false,
            show_file: true,
            show_line_number: true,
            show_level: false,
            show_thread_id: false,
            show_thread_name: true,
            #[cfg(feature = "json")]
            flatten_json: true,
            targets: BTreeMap::from([
                ("crate_a".to_string(), Verbosity::Off),
                ("crate_b::module".to_string(), Verbosity::Error),
            ]),
        };

        // When
        let actual_output = serde_yml::from_str::<TracingConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }
}
