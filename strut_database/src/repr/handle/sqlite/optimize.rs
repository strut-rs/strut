use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use strut_factory::impl_deserialize_field;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyOptimizeOnClose {
    pub(crate) enabled: bool,
    pub(crate) analysis_limit: Option<u32>,
}

const _: () = {
    impl Default for ProxyOptimizeOnClose {
        fn default() -> Self {
            Self {
                enabled: Self::default_enabled(),
                analysis_limit: Self::default_analysis_limit(),
            }
        }
    }

    impl ProxyOptimizeOnClose {
        fn default_enabled() -> bool {
            false
        }

        fn default_analysis_limit() -> Option<u32> {
            None
        }
    }
};

const _: () = {
    impl<'de> Deserialize<'de> for ProxyOptimizeOnClose {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(ProxyOptimizeOnCloseVisitor)
        }
    }

    struct ProxyOptimizeOnCloseVisitor;

    impl<'de> Visitor<'de> for ProxyOptimizeOnCloseVisitor {
        type Value = ProxyOptimizeOnClose;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean value or an integer analysis limit or a map of optimize-on-close configuration")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(ProxyOptimizeOnClose {
                enabled: value,
                ..ProxyOptimizeOnClose::default()
            })
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            let analysis_limit = if value < 0 {
                Some(0)
            } else if value > u32::MAX as i64 {
                Some(u32::MAX)
            } else {
                Some(value as u32)
            };

            Ok(ProxyOptimizeOnClose {
                enabled: true,
                analysis_limit,
            })
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            let analysis_limit = if value > u32::MAX as u64 {
                Some(u32::MAX)
            } else {
                Some(value as u32)
            };

            Ok(ProxyOptimizeOnClose {
                enabled: true,
                analysis_limit,
            })
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut enabled = None;
            let mut analysis_limit: Option<Option<u32>> = None;

            while let Some(key) = map.next_key()? {
                match key {
                    ProxyOptimizeOnCloseField::enabled => key.poll(&mut map, &mut enabled)?,
                    ProxyOptimizeOnCloseField::analysis_limit => {
                        key.poll(&mut map, &mut analysis_limit)?
                    }
                    ProxyOptimizeOnCloseField::__ignore => map.next_value()?,
                };
            }

            Ok(ProxyOptimizeOnClose {
                enabled: enabled.unwrap_or_else(ProxyOptimizeOnClose::default_enabled),
                analysis_limit: analysis_limit
                    .unwrap_or_else(ProxyOptimizeOnClose::default_analysis_limit),
            })
        }
    }

    impl_deserialize_field!(
        ProxyOptimizeOnCloseField,
        strut_deserialize::Slug::eq_as_slugs,
        enabled | is_enabled,
        analysis_limit | limit,
    );
};

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_bool() {
        // Given
        let input = r#"
true
"#;

        // When
        let actual_output = serde_yml::from_str::<ProxyOptimizeOnClose>(input).unwrap();
        let expected_output = ProxyOptimizeOnClose {
            enabled: true,
            ..ProxyOptimizeOnClose::default()
        };

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_unsigned() {
        // Given
        let input = r#"
1000
"#;

        // When
        let actual_output = serde_yml::from_str::<ProxyOptimizeOnClose>(input).unwrap();
        let expected_output = ProxyOptimizeOnClose {
            enabled: true,
            analysis_limit: Some(1000),
        };

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_signed() {
        // Given
        let input = r#"
-1000
"#;

        // When
        let actual_output = serde_yml::from_str::<ProxyOptimizeOnClose>(input).unwrap();
        let expected_output = ProxyOptimizeOnClose {
            enabled: true,
            analysis_limit: Some(0),
        };

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_overflow() {
        // Given
        let input = r#"
18446744073709551615
"#;

        // When
        let actual_output = serde_yml::from_str::<ProxyOptimizeOnClose>(input).unwrap();
        let expected_output = ProxyOptimizeOnClose {
            enabled: true,
            analysis_limit: Some(u32::MAX),
        };

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map() {
        // Given
        let input = r#"
enabled: false
limit: 0
"#;

        // When
        let actual_output = serde_yml::from_str::<ProxyOptimizeOnClose>(input).unwrap();
        let expected_output = ProxyOptimizeOnClose {
            enabled: false,
            analysis_limit: Some(0),
        };

        // Then
        assert_eq!(expected_output, actual_output);
    }
}
